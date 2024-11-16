use std::{sync::Arc, time::Duration};

use diesel::PgConnection;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use futures::stream::StreamExt;

use paho_mqtt::{
    self as mqtt, properties, AsyncClient, ConnectOptionsBuilder, PropertyCode, SubscribeOptions,
    MQTT_VERSION_5,
};

use crate::mqtt::error::MQTTError;
use crate::mqtt::handler::handle_message;
use crate::mqtt::topic::{QOS, REM_LISTENER_DISCONNECT_TOPIC, TOPICS};

pub async fn mqtt_proc(
    cli: Arc<Mutex<AsyncClient>>,
    conn: Arc<Mutex<PgConnection>>,
) -> Result<(), mqtt::Error> {
    let mut cli_lock = cli.lock().await;

    // Get message stream before connecting.
    let strm = &mut cli_lock.get_stream(25);


    let lwt = paho_mqtt::Message::new(
        REM_LISTENER_DISCONNECT_TOPIC,
        "REM listener disconnection",
        paho_mqtt::QOS_1,
    );

    // Connect with MQTT v5 and a persistent server session (no clean start).
    // For a persistent v5 session, we must set the Session Expiry Interval
    // on the server. Here we set that requests will persist for an hour
    // (3600sec) if the service disconnects or restarts.
    let conn_opts = ConnectOptionsBuilder::with_mqtt_version(MQTT_VERSION_5)
        .clean_start(false)
        .properties(properties![PropertyCode::SessionExpiryInterval => 3600])
        .will_message(lwt)
        .finalize();

    // Make the connection to the broker
    cli_lock.connect(conn_opts).await?;

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

                warn!("Warning  handling message: {:?}", err);
            }
        } else {
            // A "None" means we were disconnected. Try to reconnect...
            info!("Lost connection. Attempting reconnect.");

            let cli_lock = cli.lock().await;
            while let Err(err) = &cli_lock.reconnect().await {
                info!("Error reconnecting: {}", err);
                // For tokio use: tokio::time::delay_for()
                async_std::task::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    // Explicit return type for the async block
    Ok::<(), paho_mqtt::Error>(())
}
