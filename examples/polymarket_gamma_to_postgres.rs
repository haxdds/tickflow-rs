//! Example: Fetch active markets from Polymarket Gamma API and store in PostgreSQL.
//!
//! This example demonstrates:
//! - Fetching active markets from the Gamma API endpoint
//! - Using TickflowBuilder to create a data pipeline
//! - Storing them in PostgreSQL using the market_gamma table

#[cfg(all(feature = "polymarket", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::{
        config::AppConfig,
        connectors::polymarket::PolymarketGammaClient,
        pipeline::TickflowBuilder,
        storage::{Database, postgres_handler::polymarket::PolymarketMessageHandler},
    };
    use tracing::Level;

    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Load environment variables
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;

    // Setup database connection and schema
    let database = Database::connect(&config.database_url, PolymarketMessageHandler).await?;
    database.initialize_schema().await?;

    // Configure Polymarket Gamma API data source
    // Request delay of 200ms between paginated requests
    // Fetch markets ending on or after Dec 13, 2025
    let source = PolymarketGammaClient::new(200, "2025-12-13".to_string());

    // Start the data pipeline
    let handles = TickflowBuilder::new(source, database)
        .channel_capacity(config.channel_capacity)
        .start()
        .await?;

    // Wait for both tasks to complete
    tokio::try_join!(handles.source, handles.processor)?;
    Ok(())
}

#[cfg(not(all(feature = "polymarket", feature = "postgres")))]
fn main() {
    panic!("Enable the `polymarket` and `postgres` features to run this example.");
}
