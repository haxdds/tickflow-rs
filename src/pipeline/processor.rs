use std::sync::Arc;

use crate::core::{Message, MessageBatch, MessageSink};

pub struct MessageProcessor<M: Message> {
    sink: Arc<dyn MessageSink<M>>,
}

impl<M: Message> MessageProcessor<M> {
    pub fn new<S>(sink: S) -> Self
    where
        S: MessageSink<M>,
    {
        Self {
            sink: Arc::new(sink),
        }
    }

    pub async fn process_messages(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<MessageBatch<M>>,
    ) -> anyhow::Result<()> {
        tracing::info!("Message processor started ({})", self.sink.name());
        while let Some(batch) = rx.recv().await {
            if let Err(err) = self.sink.handle_batch(batch).await {
                tracing::warn!("{} sink error: {err}", self.sink.name());
            }
        }
        tracing::info!("Message processor stopped");
        Ok(())
    }
}
