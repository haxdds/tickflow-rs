#[cfg(all(feature = "yahoo", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::config::AppConfig;
    use tickflow::connectors::yahoo::YahooClient;
    use tickflow::pipeline::TickflowBuilder;
    use tickflow::storage::Database;
    use tickflow::storage::postgres_handler::yahoo::YahooMessageHandler;
    use tracing::Level;
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    let database = Database::connect(&config.database_url, Box::new(YahooMessageHandler)).await?;
    database.initialize_schema().await?;

    // let _sample = symbols.iter().take(3).cloned().collect();
    let _sample = ["PLTR", "AAPL"].iter().map(|s| s.to_string()).collect();
    let source = YahooClient::new(_sample, 1000);

    let handles = TickflowBuilder::new(source, database)
        .channel_capacity(1000)
        .start()
        .await?;

    tokio::try_join!(handles.source, handles.processor)?;
    Ok(())
}
