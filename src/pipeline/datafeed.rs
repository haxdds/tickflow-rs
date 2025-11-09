//! Single-producer single-consumer pipeline orchestration.

use crate::core::{Message, MessageBatch, MessageSink, MessageSource};
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;

use super::{MessageProcessor, TickflowBuilder};

/// Connects a `MessageSource` to a `MessageProcessor` via a bounded Tokio channel.
pub struct SPSCDataFeed<M, Src>
where
    M: Message,
    Src: MessageSource<M>,
{
    source: Src,
    processor: MessageProcessor<M>,
    channel_capacity: usize,
}

/// Task handles returned when an `SPSCDataFeed` is started.
pub struct SPSCDataFeedHandles {
    pub source: JoinHandle<()>,
    pub processor: JoinHandle<()>,
}

impl<M, Src> SPSCDataFeed<M, Src>
where
    M: Message,
    Src: MessageSource<M>,
{
    /// Returns a builder for configuring and launching the data feed.
    pub fn builder<Sink>(source: Src, sink: Sink) -> TickflowBuilder<M, Src, Sink>
    where
        Sink: MessageSink<M>,
    {
        TickflowBuilder::new(source, sink)
    }

    /// Creates a feed with an explicit channel capacity.
    pub fn new<Sink>(source: Src, sink: Sink, channel_capacity: usize) -> Self
    where
        Sink: MessageSink<M>,
    {
        Self {
            source,
            processor: MessageProcessor::new(sink),
            channel_capacity,
        }
    }

    /// Spawns source and processor tasks and returns their join handles.
    pub async fn start(self) -> Result<SPSCDataFeedHandles> {
        let (tx, rx) = mpsc::channel::<MessageBatch<M>>(self.channel_capacity);

        let mut source = self.source;
        let source_handle = tokio::spawn(async move {
            if let Err(err) = source.run(tx).await {
                error!("Source task failed: {err}");
            }
        });

        let processor = self.processor;
        let processor_handle = tokio::spawn(async move {
            if let Err(err) = processor.process_messages(rx).await {
                error!("Processor task failed: {err}");
            }
        });

        Ok(SPSCDataFeedHandles {
            source: source_handle,
            processor: processor_handle,
        })
    }
}
