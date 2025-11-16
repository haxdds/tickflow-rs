use tickflow::{connectors::yahoo::YahooMessage, core::{Message, MessageSink}};

// use tokio::connectors::yah
#[cfg(all(feature = "yahoo", feature = "postgres"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tickflow::connectors::yahoo::{YahooClient, YahooMessage};
    use tickflow::pipeline::TickflowBuilder;
    // use tickflow::core::MessageBatch;
    // use tokio::sync::mpsc;
    let source = YahooClient::new();
    
    let sink = StdOut::new();

    let handles = TickflowBuilder::new(source, sink)
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
