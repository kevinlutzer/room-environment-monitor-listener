use std::{
    env,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json};
use diesel::{sql_query, PgConnection, RunQueryDsl};
use paho_mqtt::AsyncClient;
use serde::Serialize;
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

/// Healthcheck handler
///
/// This handler checks the health of the application by verifying that the MQTT client is connected
/// to the broker and that the database can be queried.
async fn healthcheck_handler(State(app_state): State<AppState>) -> (StatusCode, &'static str) {
    let mqtt_client_lock = app_state.mqtt_client.lock().await;
    let mut db_lock = app_state.db.lock().await;

    let mut err_msg = "";

    // Check that the MQTT client is connected
    if !mqtt_client_lock.is_connected() {
        err_msg = "MQTT client is not connected to the broker"

    // Verify that we can query the database
    } else if sql_query("SELECT 1").execute(&mut *db_lock).is_err() {
        err_msg = "Database couldn't be connected too"
    }

    // If we get an error message, we return it as an internal error
    if !err_msg.is_empty() {
        error!(err_msg);
        return (StatusCode::INTERNAL_SERVER_ERROR, err_msg);
    }

    // Success if db query returns a result and the MQTT client is connected.
    info!("Healthcheck passed");
    (StatusCode::OK, "Ok")
}

/// VersionResponse
///
/// Contains information about the current running server version
#[derive(Serialize)]
pub struct VersionResponse {
    version: String,
    commit: String,
}

/// Version
///
/// Returns the version information of the application including a sematic version on a commit hash.
pub async fn version_handler() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        // Read versioning information from rustc environment variables at compile time
        // This variable is added as `cargo:rustc-env=VERGEN_GIT_SHA=blah` in the build.rs
        commit: std::env!("VERGEN_GIT_SHA").to_string(),
    })
}

/// Server process
///
/// This function creates the axum server and binds it to a TCP socket. This function
/// is async and blocking.
pub async fn server_proc(
    config: Arc<Settings>,
    mqtt_client: Arc<Mutex<AsyncClient>>,
    db: Arc<Mutex<PgConnection>>,
) {
    let addr = SocketAddr::new(IpAddr::V4(config.host), config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let app = axum::Router::new()
        .route("/v1/version", get(version_handler))
        .route("/v1/healthcheck", get(healthcheck_handler))
        .fallback(default_handler)
        .with_state(AppState { mqtt_client, db });

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
