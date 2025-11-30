#[cfg(all(feature = "yahoo", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::{
        config::AppConfig,
        connectors::yahoo::ProxyYahooClient,
        pipeline::TickflowBuilder,
        storage::{Database, postgres_handler::yahoo::YahooMessageHandler},
    };
    use tracing::Level;

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;

    // Setup database connection and schema
    let database = Database::connect(&config.database_url, YahooMessageHandler).await?;
    database.initialize_schema().await?;

    // Configure data source
    let symbols = ["PLTR", "AAPL"].iter().map(|s| s.to_string()).collect();
    let proxies = [
        "r2VGXNT8iGmOeYi:ofOwRXeEm9pQgO7@212.32.123.187:43160",
        "AvtZPuXpe7yV7xA:k3toqiMZudQeXS7@207.135.202.204:46128",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let source = ProxyYahooClient::new(proxies, symbols, 2000_u64)?;

    // Start the data pipeline
    let handles = TickflowBuilder::new(source, database)
        .channel_capacity(config.channel_capacity)
        .start()
        .await?;

    // Wait for both tasks to complete
    tokio::try_join!(handles.source, handles.processor)?;
    Ok(())
}

#[cfg(not(all(feature = "yahoo", feature = "postgres")))]
fn main() {
    panic!("Enable the `yahoo` and `postgres` features to run this example.");
}
