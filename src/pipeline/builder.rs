//! Builder utilities for wiring message sources to processors.

use std::marker::PhantomData;

use crate::core::{Message, MessageSink, MessageSource};

use super::{SPSCDataFeed, SPSCDataFeedHandles};

/// Fluent builder for constructing and launching an `SPSCDataFeed`.
///
/// Allows callers to start from any compatible `MessageSource`/`MessageSink` pair
/// and tweak runtime parameters such as channel capacity before spawning tasks.
pub struct TickflowBuilder<M, Src, Sink>
where
    M: Message,
    Src: MessageSource<M>,
    Sink: MessageSink<M>,
{
    source: Src,
    sink: Sink,
    channel_capacity: usize,
    _marker: PhantomData<M>,
}

impl<M, Src, Sink> TickflowBuilder<M, Src, Sink>
where
    M: Message,
    Src: MessageSource<M>,
    Sink: MessageSink<M>,
{
    /// Creates a builder with sensible defaults for the given source and sink.
    pub fn new(source: Src, sink: Sink) -> Self {
        Self {
            source,
            sink,
            channel_capacity: 1_000,
            _marker: PhantomData,
        }
    }

    /// Overrides the bounded channel capacity between source and processor stages.
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Builds an `SPSCDataFeed` without starting any asynchronous tasks.
    pub fn build(self) -> SPSCDataFeed<M, Src> {
        let Self {
            source,
            sink,
            channel_capacity,
            ..
        } = self;
        SPSCDataFeed::new(source, sink, channel_capacity)
    }

    /// Builds and starts the data feed, returning the spawned task handles.
    pub async fn start(self) -> anyhow::Result<SPSCDataFeedHandles> {
        self.build().start().await
    }
}
