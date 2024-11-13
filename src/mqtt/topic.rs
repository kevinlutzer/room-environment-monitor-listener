use paho_mqtt::QOS_1;

// The topics to which we subscribe.
pub const REM_DATA_TOPIC: &str = "rem/data";
pub const REM_STATUS_TOPIC: &str = "rem/status";

pub const TOPICS: &[&str] = &["rem/data", "rem/status"];
pub const QOS: &[i32] = &[QOS_1, QOS_1];
