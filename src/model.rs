use serde::{Deserialize, Serialize};

/// RemStatus is the structure of the status that we receive from the REM device.
#[derive(Deserialize, Serialize, Debug)]
pub struct RemStatus {
    pub id: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "uptime")]
    pub up_time: i32,
    #[serde(default)]
    pub rssi: i32,
}

/// RemData is the structure of the data that we receive from the REM device.
#[derive(Deserialize, Serialize, Debug)]
pub struct RemData {
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
