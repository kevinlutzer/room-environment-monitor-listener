use std::sync::Arc;

use paho_mqtt::Message;

use diesel::{insert_into, pg::PgConnection, prelude::*};

use tokio::sync::Mutex;
use tracing::{error, info};

use crate::mqtt::{
    message::{REMData, REMStatus},
    topic::REM_STATUS_TOPIC,
};

use crate::schema::rem_data::dsl::{
    device_id as rem_data_device_id, humidity, id as rem_data_id, pm10, pm1_0, pm2_5, pressure,
    rem_data, temperature,
};
use crate::schema::rem_status::dsl::{
    device_id as rem_status_device_id, id as rem_status_id, rem_status, up_time,
};

use super::error::MQTTError;

fn mqtt_error_from_database(e: diesel::result::Error, key: String) -> MQTTError {
    // Only error type for a duplicate key violation is violation error
    if matches!(
        e,
        diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)
    ) {
        return MQTTError::DataEntryExists(key);
    }

    return MQTTError::DatabaseError(e);
}

/// Handle the data message from the REM device and insert it into the database
async fn handle_data_message(
    conn: &Arc<Mutex<PgConnection>>,
    msg: Message,
) -> Result<String, MQTTError> {
    let data: REMData =
        serde_json::from_slice(msg.payload()).map_err(|_| MQTTError::InvalidMessage)?;

    info!("ID: {}, Device ID: {}", data.id, data.device_id);

    let a = (
        rem_data_id.eq(data.id.clone()),
        rem_data_device_id.eq(data.device_id),
        temperature.eq(data.temperature),
        pressure.eq(data.pressure),
        pm2_5.eq(data.pm2_5),
        pm1_0.eq(data.pm1_0),
        pm10.eq(data.pm10),
        humidity.eq(data.humidity),
    );

    // Lock on the Database
    let mut mut_conn = conn.lock().await;
    insert_into(rem_data)
        .values(a)
        .execute(&mut *mut_conn)
        .map_err(|e| mqtt_error_from_database(e, data.id.clone()))?;

    Ok(data.id.clone())
}

/// Handle the status message from the REM device and insert it into the database
async fn handle_status_message(
    conn: &Arc<Mutex<PgConnection>>,
    msg: Message,
) -> Result<String, MQTTError> {
    // Handle the message for rem/status
    let status: REMStatus = match serde_json::from_slice(msg.payload()) {
        Ok(s) => s,
        Err(e) => {
            info!("{:?}", msg.payload());
            error!("Error parsing status message: {:?}", e);
            return Err(MQTTError::InvalidMessage);
        }
    };

    info!(
        "ID: {}, Device ID: {}, Uptime: {}",
        status.id, status.device_id, status.up_time
    );

    let r = (
        rem_status_id.eq(status.id.clone()),
        rem_status_device_id.eq(status.device_id),
        up_time.eq(status.up_time),
    );

    // Lock on the Database
    let mut mut_conn = conn.lock().await;
    insert_into(rem_status)
        .values(r)
        .execute(&mut *mut_conn)
        .map_err(|e| mqtt_error_from_database(e, status.id.clone()))?;

    Ok(status.id.clone())
}

/// Handle the message from the MQTT server. This function will determine if the message is a status
/// message or a data message and then create the appropriate table row.
pub async fn handle_message(
    conn: &Arc<Mutex<PgConnection>>,
    msg: Message,
) -> Result<(), MQTTError> {
    // Handle the message for rem/status
    if msg.topic() == REM_STATUS_TOPIC {
        handle_status_message(conn, msg).await?
    } else {
        handle_data_message(conn, msg).await?
    };

    Ok(())
}
