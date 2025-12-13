//! Polymarket data types for market information.

use crate::core::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a Polymarket prediction market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Unique condition identifier (hex string)
    pub condition_id: String,

    /// Question identifier (hex string)
    #[serde(default)]
    pub question_id: Option<String>,

    /// URL-friendly market identifier
    #[serde(default)]
    pub market_slug: Option<String>,

    /// The market question
    #[serde(default)]
    pub question: Option<String>,

    /// Detailed description of the market
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the market is currently active
    #[serde(default)]
    pub active: bool,

    /// Whether the market has been closed
    #[serde(default)]
    pub closed: bool,

    /// Whether the market has been archived
    #[serde(default)]
    pub archived: bool,

    /// Whether the market is accepting orders
    #[serde(default)]
    pub accepting_orders: bool,

    /// Whether the order book is enabled
    #[serde(default)]
    pub enable_order_book: bool,

    /// Whether this is a negative risk market
    #[serde(default)]
    pub neg_risk: bool,

    /// ISO timestamp when the market ends
    #[serde(default)]
    pub end_date_iso: Option<String>,

    /// Game start time (for sports markets)
    #[serde(default)]
    pub game_start_time: Option<String>,

    /// Timestamp when orders started being accepted
    #[serde(default)]
    pub accepting_order_timestamp: Option<String>,

    /// Minimum order size
    #[serde(default)]
    pub minimum_order_size: f64,

    /// Minimum tick size for price movements
    #[serde(default)]
    pub minimum_tick_size: f64,

    /// Maker base fee
    #[serde(default)]
    pub maker_base_fee: f64,

    /// Taker base fee
    #[serde(default)]
    pub taker_base_fee: f64,

    /// Delay in seconds for orders
    #[serde(default)]
    pub seconds_delay: i32,

    /// Token information (stored as JSONB)
    #[serde(default)]
    pub tokens: Value,

    /// Reward configuration (stored as JSONB)
    #[serde(default)]
    pub rewards: Value,

    /// Tags associated with the market (stored as JSONB)
    #[serde(default)]
    pub tags: Value,

    /// Icon URL
    #[serde(default)]
    pub icon: Option<String>,

    /// Image URL
    #[serde(default)]
    pub image: Option<String>,

    /// FPMM contract address
    #[serde(default)]
    pub fpmm: Option<String>,

    /// Negative risk market ID
    #[serde(default)]
    pub neg_risk_market_id: Option<String>,

    /// Negative risk request ID
    #[serde(default)]
    pub neg_risk_request_id: Option<String>,

    /// Whether notifications are enabled
    #[serde(default)]
    pub notifications_enabled: bool,

    /// Whether this is a 50-50 outcome market
    #[serde(default)]
    pub is_50_50_outcome: bool,
}

/// Message types from Polymarket data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolymarketMessage {
    /// A market listing
    Market(Market),
}

impl Message for PolymarketMessage {}
