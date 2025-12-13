//! Polymarket prediction market data connector.

pub mod client;
pub mod types;

pub use client::PolymarketClient;
pub use types::{Market, PolymarketMessage};

