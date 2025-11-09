//! Message batches processor that dispatches to a sink.

use std::sync::Arc;

use crate::core::{Message, MessageBatch, MessageSink};

/// Wraps a `MessageSink` and provides async batch processing.
pub struct MessageProcessor<M: Message> {
    sink: Arc<dyn MessageSink<M>>,
}

impl<M: Message> MessageProcessor<M> {
    /// Creates a processor for the given sink, boxing it for shared ownership.
    pub fn new<S>(sink: S) -> Self
    where
        S: MessageSink<M>,
    {
        Self {
            sink: Arc::new(sink),
        }
    }

    /// Consumes messages from the provided receiver and forwards them to the sink.
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
