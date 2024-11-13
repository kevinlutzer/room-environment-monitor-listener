use std::{sync::Arc, time::Duration};

use diesel::PgConnection;
use tokio::sync::Mutex;
use tracing::info;

use futures::stream::StreamExt;

use paho_mqtt::{
    self as mqtt, properties, AsyncClient, ConnectOptionsBuilder, PropertyCode, SubscribeOptions,
    MQTT_VERSION_5,
};

use crate::mqtt::handler::handle_message;

// The topics to which we subscribe.
const TOPICS: &[&str] = &["rem/data", "rem/status"];
const QOS: &[i32] = &[1, 1];

pub async fn mqtt_proc(
    cli: Arc<Mutex<AsyncClient>>,
    conn: Arc<Mutex<PgConnection>>,
) -> Result<(), mqtt::Error> {
    let mut cli_lock = cli.lock().await;

    // Get message stream before connecting.
    let strm = &mut cli_lock.get_stream(25);

    // Define the set of options for the connection
    let lwt = paho_mqtt::Message::new(
        "test/lwt",
        "[LWT] Async subscriber v5 lost connection",
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
            handle_message(&conn, msg).await;
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
