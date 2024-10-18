use envconfig::Envconfig;
use std::net::Ipv4Addr;

// Definition of the configuration for the application.
#[derive(Envconfig)]
pub struct Settings {
    #[envconfig(from = "MQTT_HOST")]
    pub mqtt_host: String,

    #[envconfig(from = "MQTT_PORT")]
    pub mqtt_port: u16,

    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,

    #[envconfig(from = "HOST")]
    pub host: Ipv4Addr,

    #[envconfig(from = "PORT")]
    pub port: u16,
}
