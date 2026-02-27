//! Application configuration from environment variables.

use std::env;

/// Application configuration loaded from environment.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub store_name: String,
    pub otel_exporter_endpoint: Option<String>,
    pub otel_service_name: String,
}

impl Config {
    /// Load configuration from environment. Loads `.env` from the working directory if present.
    /// Environment variables take precedence over `.env` file values.
    pub fn load() -> Self {
        let _ = dotenvy::dotenv();

        let port = env::var("APP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        let store_name = env::var("STATESTORE_NAME").unwrap_or_else(|_| "statestore".to_string());

        let otel_exporter_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

        let otel_service_name =
            env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "example-rust-dapr-otel".to_string());

        Self {
            port,
            store_name,
            otel_exporter_endpoint,
            otel_service_name,
        }
    }
}
