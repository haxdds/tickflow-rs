//! Application configuration helpers.
use anyhow::{Result, anyhow};
use std::env;

/// Aggregated configuration required to run the Tickflow binary.
pub struct AppConfig {
    pub database_url: String,
    pub alpaca_api_key: String,
    pub alpaca_api_secret: String,
    pub alpaca_ws_url: String,
    pub channel_capacity: usize,
}

impl AppConfig {
    /// Builds an `AppConfig` by reading the expected environment variables.
    ///
    /// Provides defaults where reasonable (e.g. channel size) and returns an error when
    /// a required variable is missing or malformed.
    pub fn from_env() -> Result<Self> {
        let channel_capacity = env::var("DATAFEED_CHANNEL_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_000);

        let database_url = match env::var("DATABASE_URL") {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("DATABASE_URL must be set")),
        };

        let alpaca_api_key = match env::var("APCA_API_KEY_ID") {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("APCA_API_KEY_ID must be set")),
        };

        let alpaca_api_secret = match env::var("APCA_API_SECRET_KEY") {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("APCA_API_SECRET_KEY must be set")),
        };

        let alpaca_ws_url = match env::var("APCA_WS_URL") {
            Ok(val) => val,
            Err(_) => return Err(anyhow!("ALPACA_WS_URL must be set")),
        };

        Ok(Self {
            database_url,
            alpaca_api_key,
            alpaca_api_secret,
            alpaca_ws_url,
            channel_capacity,
        })
    }
}
