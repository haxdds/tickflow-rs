mod config;
mod datafeed;
mod message_types;
mod database;
mod websocket;
mod processor;
mod messages;

use anyhow::Result;
use config::AppConfig;
use database::Database;
use datafeed::SPSCDataFeed as DataFeed;
use websocket::AlpacaWebSocketClient;
use tracing_subscriber;
use dotenvy;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let database = Database::connect(&config.database_url).await?;
    database.initialize_schema().await?;

    let mut websocket =
        AlpacaWebSocketClient::new(&config.alpaca_ws_url, &config.alpaca_api_key, &config.alpaca_api_secret);

    let datafeed = DataFeed::new(websocket, database, config.channel_capacity);
    let handles = datafeed.start().await?;

    tokio::try_join!(handles.source, handles.processor)?;
    Ok(())
}