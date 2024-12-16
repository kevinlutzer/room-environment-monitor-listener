use thiserror::Error;

#[derive(Error, Debug)]
pub enum MQTTClientError {
    #[error("Database entry already exists for key: {}", .0)]
    DataEntryExists(String),
    #[error("Database error: {}", .0)]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Invalid message")]
    InvalidMessage,
}
