use std::sync::Arc;

use anyhow::Result;
use diesel::PgConnection;
use futures::stream::StreamExt;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};
use tracing::{error, info, warn};

use paho_mqtt::{AsyncClient, SubscribeOptions};

use crate::mqtt::error::MQTTError;
use crate::mqtt::handler::handle_message;
use crate::mqtt::topic::{QOS, TOPICS};

pub async fn mqtt_proc(cli: Arc<Mutex<AsyncClient>>, conn: Arc<Mutex<PgConnection>>) -> Result<()> {
    let mut cli_lock = cli.lock().await;

    // Get message stream before connecting.
    let strm = &mut cli_lock.get_stream(25);

    info!("Subscribing to topics: {:?}", TOPICS);
    let sub_opts = vec![SubscribeOptions::with_retain_as_published(); TOPICS.len()];
    cli_lock
        .subscribe_many_with_options(TOPICS, QOS, &sub_opts, None)
        .await?;

    drop(cli_lock);

    // Note that we're not providing a way to cleanly shut down and
    // disconnect. Therefore, when you kill this app (with a ^C or
    // whatever) the broker will get an unexpected drop and then
    // should emit the LWT message.
    info!("Waiting for messages...");
    while let Some(msg_opt) = strm.next().await {
        if let Some(msg) = msg_opt {
            // Just log an errors if we can't handle the message, the only real error error we care about is
            // a database error, not from a foreign key violation.
            if let Err(err) = handle_message(&conn, msg).await {
                if matches!(err, MQTTError::DatabaseError(_)) {
                    error!("Unknown database error when trying to insert new data or status message: {:?}", err);
                    continue;
                }

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
