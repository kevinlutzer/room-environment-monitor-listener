use std::sync::Arc;

use paho_mqtt::Message;

use tokio::sync::Mutex;
use tracing::{error, info};

use crate::model::{REMData, REMStatus};
use crate::mqtt::{error::MQTTClientError, topic::REM_STATUS_TOPIC};

use crate::repo::client::REMRepo;

use super::topic::REM_DATA_TOPIC;

/// Handle the message from the MQTT server. This function will determine if the message is a status
/// message or a data message and then create the appropriate table row.
pub async fn handle_message(
    repo: &Arc<Mutex<REMRepo>>,
    msg: Message,
) -> Result<(), MQTTClientError> {
    let topic = msg.topic();
    match topic {
        REM_DATA_TOPIC => {
            let data: REMData = serde_json::from_slice(msg.payload())
                .map_err(|_| MQTTClientError::InvalidMessage)?;

            info!("ID: {}, Device ID: {}", data.id, data.device_id);

            repo.lock()
                .await
                .insert_rem_data(data)
                .await
                .map_err(|f| MQTTClientError::Repo(f))
        }

        REM_STATUS_TOPIC => {
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

            repo.lock()
                .await
                .insert_rem_status(status)
                .await
                .map_err(|f| MQTTClientError::Repo(f))
        }

        _ => Err(MQTTClientError::UnsupportedMessage),
    }
}
