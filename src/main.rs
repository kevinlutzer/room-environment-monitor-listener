pub mod api;
pub mod mqtt;
pub mod schema;
pub mod settings;

use dotenv::dotenv;
use envconfig::Envconfig;
use mqtt::mqtt_proc;
use paho_mqtt::{AsyncClient, CreateOptionsBuilder};
use settings::Settings;
use std::{process, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

use diesel::{insert_into, prelude::*};

// use diesel::pg::PgConnection;

#[tokio::main]
async fn main() {
    // Load the configuration from the environment.
    dotenv().ok();

    // Setup tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = Arc::new(Settings::init_from_env().unwrap());
    let host = format!("mqtt://{}:{}", config.mqtt_host, config.mqtt_port);

    info!("Connecting to the MQTT server at '{}'...", host);

    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id("rust_async_sub_v5")
        .finalize();

    // Create the client connection
    let cli = Mutex::new(AsyncClient::new(create_opts).unwrap_or_else(|e| {
        error!("Error creating the client: {:?}", e);
        process::exit(1);
    }));

    // Create the postgres connection
    let conn = Mutex::new(
        PgConnection::establish(&config.database_url).expect("Error connecting to the database"),
    );

    // Start routine to handle mqtt messages from subscribed topics
    let _ = tokio::spawn(mqtt_proc(cli, conn, config)).await.unwrap();
}
