//! Example: Fetch all Polymarket markets and store them in PostgreSQL.
//!
//! This example demonstrates using the Polymarket connector to fetch all markets
//! via the CLOB API and store them in a PostgreSQL database.
//!
//! Required environment variables:
//! - `DATABASE_URL`: PostgreSQL connection string
//! - `PK`: Polymarket private key for API authentication
//!
//! Run with:
//! ```bash
//! cargo run --example polymarket_to_postgres --features polymarket,postgres
//! ```

#[cfg(all(feature = "polymarket", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::{
        config::AppConfig,
        connectors::polymarket::PolymarketClient,
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

    // Configure Polymarket data source
    // Request delay of 100ms between paginated requests
    let source = PolymarketClient::new(config.polymarket_private_key, 100);

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
