use tickflow::core::MessageSink;
use tickflow::connectors::yahoo::YahooMessage;

#[cfg(all(feature = "yahoo", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::connectors::yahoo::{YahooClient, YahooMessage};
    use tickflow::pipeline::TickflowBuilder;
    use tickflow::config::AppConfig;
    use tickflow::connectors::yahoo::symbols::load_symbols;
    use tickflow::storage::Database;
    use tickflow::storage::postgres_handler::yahoo::YahooMessageHandler;
    use tracing::Level;
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
    let database = Database::connect(&config.database_url, Box::new(YahooMessageHandler)).await?;
    database.initialize_schema().await?;

    let symbols = load_symbols(config.symbols_path.clone()).await?;
    // let _sample = symbols.iter().take(3).cloned().collect();
    let _sample = vec!["PLTR", "AAPL"].iter().map(|s| s.to_string() ).collect();
    let source = YahooClient::new(_sample, 1000);


    
    let handles = TickflowBuilder::new(source, database)
        .channel_capacity(1000)
        .start()
        .await?;

    tokio::try_join!(handles.source, handles.processor)?;
    Ok(())
}


struct StdOut {

}

impl MessageSink<YahooMessage> for StdOut {
    fn name(&self) -> &'static str {
        "StdOut"
    }

    fn handle_batch<'a>(
        &'a self,
        batch: tickflow::prelude::MessageBatch<YahooMessage>,
    ) -> std::pin::Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> 
    {
        Box::pin(async move {
            for item in &batch {
                println!("{}", item);
            }
            Ok(())
        })
    }
}

impl StdOut {
    pub fn new() -> Self {
        Self {}
    }
}
