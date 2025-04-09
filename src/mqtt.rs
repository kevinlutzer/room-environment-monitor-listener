use std::sync::Arc;

use anyhow::Result;
use futures::stream::StreamExt;
use thiserror::Error;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};
use tracing::{error, info, warn};

use paho_mqtt::{AsyncClient, Message, SubscribeOptions};

use crate::topic::{REM_DATA_TOPIC, REM_STATUS_TOPIC};
use crate::{
    model::{RemData, RemStatus},
    repo::RemRepoError,
};
use crate::{
    repo::RemRepo,
    topic::{QOS, SUBSCRIBED_TOPICS},
};

#[derive(Error, Debug)]
pub enum MQTTClientError {
    #[error("Database entry already exists for key: {}", .0)]
    DataEntryExists(String),
    #[error("Database error: {}", .0)]
    Repo(#[from] RemRepoError),
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Unsupported message type: {}", .0)]
    UnsupportedMessage(String),
}

/// Handle the message from the MQTT server. This function will determine if the message is a status
/// message or a data message and then create the appropriate table row.
async fn handle_message(repo: &Arc<Mutex<RemRepo>>, msg: Message) -> Result<(), MQTTClientError> {
    let topic = msg.topic();
    match topic {
        REM_DATA_TOPIC => {
            let data: RemData = serde_json::from_slice(msg.payload())
                .map_err(|_| MQTTClientError::InvalidMessage)?;

            info!("ID: {}, Device ID: {}", data.id, data.device_id);

            repo.lock()
                .await
                .insert_rem_data(data)
                .await
                .map_err(MQTTClientError::Repo)
        }

        REM_STATUS_TOPIC => {
            let status: RemStatus = match serde_json::from_slice(msg.payload()) {
                Ok(s) => s,
                Err(e) => {
                    info!("{:?}", msg.payload());
                    error!("Error parsing status message: {:?}", e);
                    return Err(MQTTClientError::InvalidMessage);
                }
            };

            info!(
                "ID: {}, Device ID: {}, Uptime: {}",
                status.id, status.device_id, status.up_time
            );

            repo.lock()
                .await
                .insert_rem_status(status)
                .await
                .map_err(MQTTClientError::Repo)
        }

        _ => Err(MQTTClientError::UnsupportedMessage(topic.to_string())),
    }
}

pub async fn mqtt_proc(cli: Arc<Mutex<AsyncClient>>, repo: Arc<Mutex<RemRepo>>) -> Result<()> {
    let mut cli_lock = cli.lock().await;

    // Get message stream before connecting.
    let strm = &mut cli_lock.get_stream(25);

    info!("Subscribing to topics: {:?}", SUBSCRIBED_TOPICS);
    let sub_opts = vec![SubscribeOptions::with_retain_as_published(); SUBSCRIBED_TOPICS.len()];
    cli_lock
        .subscribe_many_with_options(SUBSCRIBED_TOPICS, QOS, &sub_opts, None)
        .await?;

    drop(cli_lock);

    // Note that we're not providing a way to cleanly shut down and
    // disconnect. Therefore, when you kill this app (with a ^C or
    // whatever) the broker will get an unexpected drop and then
    // should emit the LWT message.
    info!("Waiting for messages...");
    while let Some(msg_opt) = strm.next().await {
        if let Some(msg) = msg_opt {
            info!("Received message: {:?}", msg.clone());

            // Just log an errors if we can't handle the message, the only real error error we care about is
            // a database error, not from a foreign key violation.
            if let Err(err) = handle_message(&repo, msg).await {
                warn!("Warning handling message: {:?}", err);
            }
        } else {
            // A "None" means we were disconnected. Try to reconnect...
            warn!("Lost connection. Attempting reconnect.");

            let cli_lock = cli.lock().await;
            while let Err(err) = &cli_lock.reconnect().await {
                error!("Error reconnecting: {}", err);
                sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    Ok(())
}
