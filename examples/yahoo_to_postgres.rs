#[cfg(all(feature = "yahoo", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::{
        config::AppConfig,
        connectors::yahoo::YahooClient,
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
    let database = Database::connect(&config.database_url, Box::new(YahooMessageHandler)).await?;
    database.initialize_schema().await?;

    // Configure data source
    let symbols = ["PLTR", "AAPL"].iter().map(|s| s.to_string()).collect();
    let batch_size = 1000;
    let source = YahooClient::new(symbols, batch_size);

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
