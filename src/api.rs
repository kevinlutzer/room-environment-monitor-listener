use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get};
use diesel::{sql_query, PgConnection, RunQueryDsl};
use paho_mqtt::AsyncClient;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::settings::Settings;

#[derive(Clone)]
struct AppState {
    mqtt_client: Arc<Mutex<AsyncClient>>,
    db: Arc<Mutex<PgConnection>>,
}

async fn default_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

async fn healthcheck_handler(State(app_state): State<AppState>) -> (StatusCode, &'static str) {
    let mqtt_client_lock = app_state.mqtt_client.lock().await;
    let mut db_lock = app_state.db.lock().await;

    // Check that the MQTT client is connected
    let err_msg = if !mqtt_client_lock.is_connected() {
        "MQTT client is not connected to the broker"
    // Verify that we can query the database
    } else if sql_query("SELECT 1").execute(&mut *db_lock).is_err() {
        "Database couldn't be connected too"
    } else {
        ""
    };

    // else if db_lock.batch_execute("SELECT 1").is_err() {
    //     "Database couldn't be connected too"
    // }
    // else {
    //     ""
    // };

    if !err_msg.is_empty() {
        error!(err_msg);
        return (StatusCode::INTERNAL_SERVER_ERROR, err_msg);
    }

    info!("Healthcheck passed");
    (StatusCode::OK, "Ok")
}

pub async fn server_proc(
    config: Arc<Settings>,
    mqtt_client: Arc<Mutex<AsyncClient>>,
    db: Arc<Mutex<PgConnection>>,
) {
    let addr = SocketAddr::new(IpAddr::V4(config.host), config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let app = axum::Router::new()
        .route("/v1/healthcheck", get(healthcheck_handler))
        .fallback(default_handler)
        .with_state(AppState { mqtt_client, db });

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
