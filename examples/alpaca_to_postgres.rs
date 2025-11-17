#[cfg(all(feature = "alpaca", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::config::AppConfig;
    use tickflow::connectors::alpaca::websocket::AlpacaWebSocketClient;
    use tickflow::prelude::*;
    use tickflow::storage::Database;
    use tickflow::storage::postgres_handler::alpaca::AlpacaMessageHandler;
    use tracing::Level;

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;

    let database = Database::connect(&config.database_url, AlpacaMessageHandler).await?;
    database.initialize_schema().await?;

    let websocket = AlpacaWebSocketClient::new(
        &config.alpaca_ws_url,
        &config.alpaca_api_key,
        &config.alpaca_api_secret,
        &[],
        &["ETH/USD"],
        &[],
    );

    let handles = TickflowBuilder::new(websocket, database)
        .channel_capacity(config.channel_capacity)
        .start()
        .await?;

    tokio::try_join!(handles.source, handles.processor)?;
    Ok(())
}

#[cfg(not(all(feature = "alpaca", feature = "postgres")))]
fn main() {
    panic!("Enable the `alpaca` and `postgres` features to run this example.");
}
