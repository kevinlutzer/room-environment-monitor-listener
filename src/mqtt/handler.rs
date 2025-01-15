use std::sync::Arc;

use paho_mqtt::Message;

use diesel::{insert_into, pg::PgConnection, prelude::*};

use tokio::sync::Mutex;
use tracing::{error, info};
use model::{REMData, REMStatus};

use crate::mqtt::{
    error::MQTTClientError,
    topic::REM_STATUS_TOPIC,
};

use crate::repo::client::REMRepo;
use crate::schema::rem_data::dsl::{
    device_id as rem_data_device_id, humidity, id as rem_data_id, pm10, pm1_0, pm2_5, pressure,
    rem_data, temperature,
};
use crate::schema::rem_status::dsl::{
    device_id as rem_status_device_id, id as rem_status_id, rem_status, up_time,
};

/// Handle the data message from the REM device and insert it into the database
async fn handle_data_message(
    conn: &Arc<Mutex<PgConnection>>,
    repo: &Arc<Mutex<REMRepo>>,
    msg: Message,
) -> Result<String, ()> {
    let data: REMData =
        serde_json::from_slice(msg.payload()).map_err(|_| MQTTClientError::InvalidMessage)?;

    info!("ID: {}, Device ID: {}", data.id, data.device_id);
 
    repo.lock().await.insert_rem_data(data).await;

    Ok(data.id.clone())
}

/// Handle the status message from the REM device and insert it into the database
async fn handle_status_message(
    conn: &Arc<Mutex<PgConnection>>,
    repo: &Arc<Mutex<REMRepo>>,
    msg: Message,
) -> Result<(), MQTTClientError> {
    // Handle the message for rem/status
    let status: REMStatus = match serde_json::from_slice(msg.payload()) {
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

    return repo.lock().await.insert_rem_status(status).await;
}

/// Handle the message from the MQTT server. This function will determine if the message is a status
/// message or a data message and then create the appropriate table row.
pub async fn handle_message(
    conn: &Arc<Mutex<PgConnection>>,
    repo: &Arc<Mutex<REMRepo>>,
    msg: Message,
) -> Result<(), MQTTClientError> {
    // Handle the message for rem/status
    if msg.topic() == REM_STATUS_TOPIC {
        return handle_status_message(conn, repo, msg).await;
    } 

    return handle_data_message(conn, repo, msg).await;
}
