//! This module contains the configuration settings for the application.
/// It uses the `envconfig` crate to load settings from environment variables.
/// The settings include the MQTT host and port, the database URL, the server host and port.
use envconfig::Envconfig;
use std::net::Ipv4Addr;

/// Definition of the configuration for the application.
#[derive(Envconfig)]
pub struct Settings {
    /// The hostname of the MQTT server. This should be a valid hostname or IP address.
    #[envconfig(from = "MQTT_HOST")]
    pub mqtt_host: String,

    /// The port to connect to the MQTT server.
    #[envconfig(from = "MQTT_PORT")]
    pub mqtt_port: u16,

    /// Database url must be in the format of `postgres://user:password@host:port/dbname`
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,

    /// IP address used of the API server for the application
    #[envconfig(from = "HOST")]
    pub host: Ipv4Addr,

    /// Port used of the API server for the application
    #[envconfig(from = "PORT")]
    pub port: u16,
}
