use std::marker::PhantomData;

use crate::core::{Message, MessageSink, MessageSource};

use super::{SPSCDataFeed, SPSCDataFeedHandles};

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
    pub fn new(source: Src, sink: Sink) -> Self {
        Self {
            source,
            sink,
            channel_capacity: 1_000,
            _marker: PhantomData,
        }
    }

    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    pub fn build(self) -> SPSCDataFeed<M, Src> {
        let Self {
            source,
            sink,
            channel_capacity,
            ..
        } = self;
        SPSCDataFeed::new(source, sink, channel_capacity)
    }

    pub async fn start(self) -> anyhow::Result<SPSCDataFeedHandles> {
        self.build().start().await
    }
}
