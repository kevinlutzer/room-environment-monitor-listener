use serde::Deserialize;

/// REMStatus is the structure of the status that we receive from the REM device.
#[derive(Deserialize)]
pub struct REMStatus {
    pub id: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "uptime")]
    pub up_time: i32,
}

/// REMData is the structure of the data that we receive from the REM device.
#[derive(Deserialize)]
pub struct REMData {
    pub id: String,

    #[serde(rename = "deviceId")]
    pub device_id: String,

    pub pm2_5: f32,
    pub pm1_0: f32,
    pub pm10: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,

    #[serde(rename = "vocIndex")]
    pub voc_index: f32,
}
