pub mod api;
pub mod model;
pub mod mqtt;
pub mod repo;
pub mod schema;
pub mod settings;

use api::server_proc;
use mqtt::{client::mqtt_proc, topic::REM_LISTENER_DISCONNECT_TOPIC};
use repo::client::REMRepo;
use settings::Settings;

use dotenv::dotenv;
use envconfig::Envconfig;
use paho_mqtt::{
    properties, AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, PropertyCode,
    MQTT_VERSION_5,
};
use std::{
    process::{self, exit},
    sync::Arc,
    time::Duration,
};

use tokio::{join, sync::Mutex};
use tracing::{debug, error, info};

use diesel::prelude::*;

const MQTT_CLIENT_ID: &str = "room-environment-client-listener";
const MQTT_CLIENT_FAILED_CONNECTION_ERR: i32 = 4;
const MQTT_CLIENT_FAILED_SETUP_ERR: i32 = 3;
const POSTGRES_CONNECTION_ERR: i32 = 5;

#[tokio::main]
async fn main() {
    // Load the configuration from the environment.
    dotenv().ok();

    // Setup tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    #[allow(clippy::unwrap_used)]
    // Safe to unwrap because we know that the environment settings will exist
    let config = Arc::new(Settings::init_from_env().unwrap());
    let host = format!("mqtt://{}:{}", config.mqtt_host, config.mqtt_port);

    info!("Connecting to the MQTT server at '{}'...", host);

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(MQTT_CLIENT_ID)
        .finalize();

    // Create the client connection
    let mqtt_client_mutex = Arc::new(Mutex::new(AsyncClient::new(create_opts).unwrap_or_else(
        |e| {
            error!("Error creating the client: {:?}", e);
            process::exit(MQTT_CLIENT_FAILED_SETUP_ERR);
        },
    )));

    // Message to return to the client if we suddenly loose connector, or we shut
    // down the listener
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

    // Make the connection to the broker, if we fail keep trying until we connect
    // theoretically the MQTT service could not have been booted yet. Retry for 10 seconds
    // before restarting the entire service.
    let mut count = 0;
    let mqtt_client_lock = mqtt_client_mutex.lock().await;
    loop {
        if let Err(e) = mqtt_client_lock.connect(conn_opts.clone()).await {
            error!(
                "Failed to connect to the MQTT client with this error: {}",
                e
            );
            if count == 10 {
                error!("Exiting out, not retrying anymore");
                exit(MQTT_CLIENT_FAILED_CONNECTION_ERR)
            }

            debug!("Waiting a second and then retrying, on attempt: {}", count);
            tokio::time::sleep(Duration::from_millis(1000)).await;

            count += 1;
            continue;
        }

        // Exit if we get no error
        break;
    }

    info!("Creating connection to the postgres client ...");

    // Create the postgres connection
    let pg_connection = PgConnection::establish(&config.database_url);
    if let Err(e) = pg_connection {
        error!(
            "Failed to setup the connection to the postgres instance: {}",
            e
        );
        exit(POSTGRES_CONNECTION_ERR);
    }

    // Safe to unwrap because we previously checked the error
    #[allow(clippy::unwrap_used)]
    let pg_connection_mutex: Arc<Mutex<PgConnection>> =
        Arc::new(Mutex::new(pg_connection.unwrap()));

    let repo = Arc::new(Mutex::new(REMRepo::new(pg_connection_mutex.clone())));

    // Start routine to handle mqtt messages from subscribed topics
    let _ = join!(
        tokio::spawn(mqtt_proc(mqtt_client_mutex.clone(), repo.clone())),
        tokio::spawn(server_proc(
            config,
            mqtt_client_mutex.clone(),
            pg_connection_mutex.clone(),
            repo
        ))
    );
}
