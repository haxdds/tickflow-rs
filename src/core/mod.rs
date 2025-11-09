//! Core messaging traits and type aliases shared across Tickflow components.

mod traits;

pub use traits::{Message, MessageBatch, MessageSink, MessageSource};
