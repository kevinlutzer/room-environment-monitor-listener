use serde::Deserialize;

#[derive(Deserialize)]
pub struct REMStatus {
    pub id: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "uptime")]
    pub up_time: i32,
}

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
}