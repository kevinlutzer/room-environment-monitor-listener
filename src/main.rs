pub mod schema;
pub mod settings;

use diesel::{insert_into, prelude::*};
use dotenv::dotenv;
use envconfig::Envconfig;
use futures::{executor::block_on, stream::StreamExt};
use paho_mqtt::{self as mqtt, MQTT_VERSION_5};
use schema::rem_status::dsl::*;
use serde::Deserialize;
use settings::Settings;
use std::{process, time::Duration};
use tracing::{error, info};

// The topics to which we subscribe.
const TOPICS: &[&str] = &["rem/data", "rem/status"];
const QOS: &[i32] = &[1, 1];

#[derive(Deserialize)]
struct REMStatus {
    id: String,
    #[serde(rename = "deviceId")]
    device_id: String,
    #[serde(rename = "uptime")]
    up_time: i32,
}

fn main() {
    // Load the configuration from the environment.
    dotenv().ok();

    // Setup tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = Settings::init_from_env().unwrap();
    let host = format!("mqtt://{}:{}", config.mqtt_host, config.mqtt_port);

    info!("Connecting to the MQTT server at '{}'...", host);

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("rust_async_sub_v5")
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        error!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    // Create the postgres connection
    let mut conn = PgConnection::establish(config.database_url.as_str())
        .expect("Error connecting to the database");

    if let Err(err) = block_on(async {
        // Get message stream before connecting.
        let mut strm = cli.get_stream(25);

        // Define the set of options for the connection
        let lwt = mqtt::Message::new(
            "test/lwt",
            "[LWT] Async subscriber v5 lost connection",
            mqtt::QOS_1,
        );

        // Connect with MQTT v5 and a persistent server session (no clean start).
        // For a persistent v5 session, we must set the Session Expiry Interval
        // on the server. Here we set that requests will persist for an hour
        // (3600sec) if the service disconnects or restarts.
        let conn_opts = mqtt::ConnectOptionsBuilder::with_mqtt_version(MQTT_VERSION_5)
            .clean_start(false)
            .properties(mqtt::properties![mqtt::PropertyCode::SessionExpiryInterval => 3600])
            .will_message(lwt)
            .finalize();

        // Make the connection to the broker
        cli.connect(conn_opts).await?;

        info!("Subscribing to topics: {:?}", TOPICS);
        let sub_opts = vec![mqtt::SubscribeOptions::with_retain_as_published(); TOPICS.len()];
        cli.subscribe_many_with_options(TOPICS, QOS, &sub_opts, None)
            .await?;

        // Just loop on incoming messages.
        info!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                if msg.retained() {
                    info!("(R) ");
                }
                info!("{}", msg);
                
                // Handle the message
                if msg.topic() == "rem/status" {
                    let status: REMStatus = serde_json::from_slice(msg.payload()).unwrap();
                    info!("Device ID: {}, Uptime: {}", status.device_id, status.up_time);

                    insert_into(rem_status)
                        .values((id.eq(status.id), device_id.eq(status.device_id), up_time.eq(status.up_time)))
                        .execute(&mut conn)
                        .unwrap();
                }
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                info!("Lost connection. Attempting reconnect.");
                while let Err(err) = cli.reconnect().await {
                    info!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        // Explicit return type for the async block
        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}
