//! Tickflow prelude: commonly used traits re-exported for convenience.

pub use crate::core::{Message, MessageBatch, MessageSink, MessageSource};
pub use crate::pipeline::{MessageProcessor, SPSCDataFeed, SPSCDataFeedHandles, TickflowBuilder};
