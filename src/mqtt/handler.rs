use std::sync::Arc;

use paho_mqtt::Message;

use diesel::{insert_into, pg::PgConnection, prelude::*};

use tokio::sync::Mutex;
use tracing::info;

use crate::mqtt::message::REMStatus;
use crate::schema::rem_status::dsl::*;

pub async fn handle_message(conn: &Arc<Mutex<PgConnection>>, msg: Message) {
    // Lock on the Database
    let mut mut_conn = conn.lock().await;

    // Handle the message
    if msg.topic() == "rem/status" {
        let status: REMStatus = serde_json::from_slice(msg.payload()).unwrap();
        info!(
            "Device ID: {}, Uptime: {}",
            status.device_id, status.up_time
        );

        insert_into(rem_status)
            .values((
                id.eq(status.id),
                device_id.eq(status.device_id),
                up_time.eq(status.up_time),
            ))
            .execute(&mut *mut_conn)
            .unwrap();
    }
}
