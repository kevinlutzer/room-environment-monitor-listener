use crate::repo::error::REMRepoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MQTTClientError {
    #[error("Database entry already exists for key: {}", .0)]
    DataEntryExists(String),
    #[error("Database error: {}", .0)]
    Repo(#[from] REMRepoError),
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Unsupported message type: {}", .0)]
    UnsupportedMessage(String),
}
