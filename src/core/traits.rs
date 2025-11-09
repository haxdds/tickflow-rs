use std::future::Future;
use std::pin::Pin;

/// Marker trait for Tickflow message types.
pub trait Message: Send + Sync + Clone + 'static {}

/// Batch of messages processed together.
pub type MessageBatch<M> = Vec<M>;

/// Trait for sinks that handle batches of messages asynchronously.
pub trait MessageSink<M: Message>: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn handle_batch<'a>(
        &'a self,
        batch: MessageBatch<M>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}

/// Trait for sources that produce batches of messages asynchronously.
pub trait MessageSource<M: Message>: Send + 'static {
    fn run<'a>(
        &'a mut self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<M>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}
