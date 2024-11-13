use std::sync::Arc;

use paho_mqtt::Message;

use diesel::{insert_into, pg::PgConnection, prelude::*};

use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::mqtt::message::{REMData, REMStatus};
use crate::mqtt::topic::REM_STATUS_TOPIC;
use crate::schema::rem_data::dsl::{
    device_id as rem_data_device_id, humidity, id as rem_data_id, pm10, pm1_0, pm2_5, pressure,
    rem_data, temperature,
};
use crate::schema::rem_status::dsl::{
    device_id as rem_status_device_id, id as rem_status_id, rem_status, up_time,
};

pub async fn handle_message(conn: &Arc<Mutex<PgConnection>>, msg: Message) {
    // Lock on the Database
    let mut mut_conn = conn.lock().await;

    // Handle the message for rem/status
    let (key, res) = if msg.topic() == REM_STATUS_TOPIC {
        let status: REMStatus = match serde_json::from_slice(msg.payload()) {
            Ok(s) => s,
            Err(e) => {
                info!("{:?}", msg.payload());
                error!("Error parsing status message: {:?}", e);
                return;
            }
        };

        info!(
            "ID: {}, Device ID: {}, Uptime: {}",
            status.id, status.device_id, status.up_time
        );

        (
            status.id.clone(),
            insert_into(rem_status)
                .values((
                    rem_status_id.eq(status.id),
                    rem_status_device_id.eq(status.device_id),
                    up_time.eq(status.up_time),
                ))
                .execute(&mut *mut_conn),
        )
    } else {
        let data: REMData = match serde_json::from_slice(msg.payload()) {
            Ok(s) => s,
            Err(e) => {
                info!("{:?}", msg.payload());
                error!("Error parsing status message: {:?}", e);
                return;
            }
        };

        info!("ID: {}, Device ID: {}", data.id, data.device_id);

        (
            data.id.clone(),
            insert_into(rem_data)
                .values((
                    rem_data_id.eq(data.id),
                    rem_data_device_id.eq(data.device_id),
                    temperature.eq(data.temperature),
                    pressure.eq(data.pressure),
                    pm2_5.eq(data.pm2_5),
                    pm1_0.eq(data.pm1_0),
                    pm10.eq(data.pm10),
                    humidity.eq(data.humidity),
                ))
                .execute(&mut *mut_conn),
        )
    };

    if let Err(e) = res {
        // Check if database error
        if matches!(
            e,
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            )
        ) {
            warn!("Key {} already exists", key);

            return;
        }

        error!("Error inserting into database: {:?}", e);
    }
}
