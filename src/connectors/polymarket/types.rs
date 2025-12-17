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

/// Represents a market from the Gamma API endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketGamma {
    /// Unique market identifier
    pub id: String,

    /// Question being predicted
    pub question: String,

    /// Unique condition identifier (hex string)
    #[serde(rename = "conditionId")]
    pub condition_id: String,

    /// URL-friendly market identifier
    pub slug: String,

    /// Resolution source
    #[serde(default, rename = "resolutionSource")]
    pub resolution_source: Option<String>,

    /// End date in ISO format
    #[serde(rename = "endDate")]
    pub end_date: String,

    /// Liquidity amount
    #[serde(default)]
    pub liquidity: Option<String>,

    /// Start date in ISO format
    #[serde(rename = "startDate")]
    pub start_date: String,

    /// Image URL
    #[serde(default)]
    pub image: Option<String>,

    /// Icon URL
    #[serde(default)]
    pub icon: Option<String>,

    /// Description of the market
    #[serde(default)]
    pub description: Option<String>,

    /// Possible outcomes (stored as JSON array string)
    #[serde(default)]
    pub outcomes: Option<String>,

    /// Outcome prices (stored as JSON array string)
    #[serde(rename = "outcomePrices", default)]
    pub outcome_prices: Option<String>,

    /// Trading volume
    #[serde(default)]
    pub volume: Option<String>,

    /// Whether the market is active
    #[serde(default)]
    pub active: bool,

    /// Whether the market is closed
    #[serde(default)]
    pub closed: bool,

    /// Market maker address
    #[serde(rename = "marketMakerAddress", default)]
    pub market_maker_address: Option<String>,

    /// Creation timestamp
    #[serde(rename = "createdAt")]
    pub created_at: String,

    /// Last update timestamp
    #[serde(rename = "updatedAt")]
    pub updated_at: String,

    /// Whether this is a new market
    #[serde(default)]
    pub new: bool,

    /// Whether this is a featured market
    #[serde(default)]
    pub featured: bool,

    /// Submitted by address
    #[serde(rename = "submitted_by", default)]
    pub submitted_by: Option<String>,

    /// Whether the market is archived
    #[serde(default)]
    pub archived: bool,

    /// Resolved by address
    #[serde(rename = "resolvedBy", default)]
    pub resolved_by: Option<String>,

    /// Whether the market is restricted
    #[serde(default)]
    pub restricted: bool,

    /// Group item title
    #[serde(rename = "groupItemTitle", default)]
    pub group_item_title: Option<String>,

    /// Group item threshold
    #[serde(rename = "groupItemThreshold", default)]
    pub group_item_threshold: Option<String>,

    /// Question ID (hex string)
    #[serde(rename = "questionID", default)]
    pub question_id: Option<String>,

    /// Whether order book is enabled
    #[serde(rename = "enableOrderBook", default)]
    pub enable_order_book: bool,

    /// Minimum tick size for orders
    #[serde(rename = "orderPriceMinTickSize", default)]
    pub order_price_min_tick_size: Option<f64>,

    /// Minimum order size
    #[serde(rename = "orderMinSize", default)]
    pub order_min_size: Option<f64>,

    /// Volume as number
    #[serde(rename = "volumeNum", default)]
    pub volume_num: Option<f64>,

    /// Liquidity as number
    #[serde(rename = "liquidityNum", default)]
    pub liquidity_num: Option<f64>,

    /// End date in ISO format (date only)
    #[serde(rename = "endDateIso", default)]
    pub end_date_iso: Option<String>,

    /// Start date in ISO format (date only)
    #[serde(rename = "startDateIso", default)]
    pub start_date_iso: Option<String>,

    /// Whether dates have been reviewed
    #[serde(rename = "hasReviewedDates", default)]
    pub has_reviewed_dates: bool,

    /// 24 hour volume
    #[serde(rename = "volume24hr", default)]
    pub volume_24hr: Option<f64>,

    /// 1 week volume
    #[serde(rename = "volume1wk", default)]
    pub volume_1wk: Option<f64>,

    /// 1 month volume
    #[serde(rename = "volume1mo", default)]
    pub volume_1mo: Option<f64>,

    /// 1 year volume
    #[serde(rename = "volume1yr", default)]
    pub volume_1yr: Option<f64>,

    /// CLOB token IDs (stored as JSONB)
    #[serde(rename = "clobTokenIds", default)]
    pub clob_token_ids: Option<String>,

    /// UMA bond
    #[serde(rename = "umaBond", default)]
    pub uma_bond: Option<String>,

    /// UMA reward
    #[serde(rename = "umaReward", default)]
    pub uma_reward: Option<String>,

    /// 24 hour CLOB volume
    #[serde(rename = "volume24hrClob", default)]
    pub volume_24hr_clob: Option<f64>,

    /// 1 week CLOB volume
    #[serde(rename = "volume1wkClob", default)]
    pub volume_1wk_clob: Option<f64>,

    /// 1 month CLOB volume
    #[serde(rename = "volume1moClob", default)]
    pub volume_1mo_clob: Option<f64>,

    /// 1 year CLOB volume
    #[serde(rename = "volume1yrClob", default)]
    pub volume_1yr_clob: Option<f64>,

    /// Total CLOB volume
    #[serde(rename = "volumeClob", default)]
    pub volume_clob: Option<f64>,

    /// CLOB liquidity
    #[serde(rename = "liquidityClob", default)]
    pub liquidity_clob: Option<f64>,

    /// Whether accepting orders
    #[serde(rename = "acceptingOrders", default)]
    pub accepting_orders: bool,

    /// Whether negative risk
    #[serde(rename = "negRisk", default)]
    pub neg_risk: bool,

    /// Events associated with the market (stored as JSONB)
    #[serde(default)]
    pub events: Value,

    /// Whether the market is ready
    #[serde(default)]
    pub ready: bool,

    /// Whether the market is funded
    #[serde(default)]
    pub funded: bool,

    /// Accepting orders timestamp
    #[serde(rename = "acceptingOrdersTimestamp", default)]
    pub accepting_orders_timestamp: Option<String>,

    /// Whether CYOM (Create Your Own Market)
    #[serde(default)]
    pub cyom: bool,

    /// Competitive score
    #[serde(default)]
    pub competitive: Option<f64>,

    /// Whether PagerDuty notification is enabled
    #[serde(rename = "pagerDutyNotificationEnabled", default)]
    pub pager_duty_notification_enabled: bool,

    /// Whether approved
    #[serde(default)]
    pub approved: bool,

    /// Minimum size for rewards
    #[serde(rename = "rewardsMinSize", default)]
    pub rewards_min_size: Option<f64>,

    /// Maximum spread for rewards
    #[serde(rename = "rewardsMaxSpread", default)]
    pub rewards_max_spread: Option<f64>,

    /// Current spread
    #[serde(default)]
    pub spread: Option<f64>,

    /// One day price change
    #[serde(rename = "oneDayPriceChange", default)]
    pub one_day_price_change: Option<f64>,

    /// One week price change
    #[serde(rename = "oneWeekPriceChange", default)]
    pub one_week_price_change: Option<f64>,

    /// One month price change
    #[serde(rename = "oneMonthPriceChange", default)]
    pub one_month_price_change: Option<f64>,

    /// Last trade price
    #[serde(rename = "lastTradePrice", default)]
    pub last_trade_price: Option<f64>,

    /// Best bid price
    #[serde(rename = "bestBid", default)]
    pub best_bid: Option<f64>,

    /// Best ask price
    #[serde(rename = "bestAsk", default)]
    pub best_ask: Option<f64>,

    /// Whether automatically active
    #[serde(rename = "automaticallyActive", default)]
    pub automatically_active: bool,

    /// Whether to clear book on start
    #[serde(rename = "clearBookOnStart", default)]
    pub clear_book_on_start: bool,

    /// Whether manual activation is required
    #[serde(rename = "manualActivation", default)]
    pub manual_activation: bool,

    /// Whether negative risk other
    #[serde(rename = "negRiskOther", default)]
    pub neg_risk_other: bool,

    /// UMA resolution statuses (stored as JSONB)
    #[serde(rename = "umaResolutionStatuses", default)]
    pub uma_resolution_statuses: Option<String>,

    /// Whether pending deployment
    #[serde(rename = "pendingDeployment", default)]
    pub pending_deployment: bool,

    /// Whether currently deploying
    #[serde(default)]
    pub deploying: bool,

    /// Whether RFQ enabled
    #[serde(rename = "rfqEnabled", default)]
    pub rfq_enabled: bool,

    /// Whether holding rewards enabled
    #[serde(rename = "holdingRewardsEnabled", default)]
    pub holding_rewards_enabled: bool,

    /// Whether fees enabled
    #[serde(rename = "feesEnabled", default)]
    pub fees_enabled: bool,
}

/// Message types from Polymarket data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolymarketMessage {
    /// A market listing
    Market(Market),
    /// A market from Gamma API
    MarketGamma(MarketGamma),
}

impl Message for PolymarketMessage {}
