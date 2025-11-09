//! Alpaca data connector primitives.
//! Currently re-exporting the existing WebSocket client and message types.

pub mod types;
pub mod websocket;

pub use types::{AlpacaMessage, Bar, Quote, Trade};
pub use websocket::AlpacaWebSocketClient;
