//! Polymarket prediction market data connector.

pub mod client;
pub mod types;

pub use client::{PolymarketClient, PolymarketGammaClient};
pub use types::{Market, MarketGamma, PolymarketMessage};
