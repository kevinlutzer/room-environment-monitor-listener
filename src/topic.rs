use paho_mqtt::QOS_1;

/// The topic we are subscribing to, to get REM data. This includes telemetry data
/// for the REM devices like temperature, humidity, VOC index, PPM measurements for different
/// sizes of particles, etc.
pub const REM_DATA_TOPIC: &str = "rem/data";

/// The topic we receive information about the REM device status. The REM devices send the
/// status data like a heartbeat.
pub const REM_STATUS_TOPIC: &str = "rem/status";

/// Topic that the MQTT listener in this project sends a disconnect message too.
pub const REM_LISTENER_DISCONNECT_TOPIC: &str = "rem/lwt";

/// Topics that we are going to subscribe too as a listeners.
pub const SUBSCRIBED_TOPICS: &[&str] = &["rem/data", "rem/status"];

/// QOS values for the subscribed topics, note that SUBSCRIBED_TOPICS and QOS
/// need to be the same length value.
pub const QOS: &[i32] = &[QOS_1, QOS_1];
