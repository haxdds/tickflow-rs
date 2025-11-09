use serde::Deserialize;

use crate::core::Message;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "T", rename_all = "lowercase")]
pub enum AlpacaMessage {
    #[serde(rename = "success")]
    Success { msg: String },

    #[serde(rename = "error")]
    Error { code: u16, msg: String },

    #[serde(rename = "subscription")]
    #[serde(rename_all = "camelCase")]
    Subscription {
        #[serde(default)]
        trades: Vec<String>,
        #[serde(default)]
        quotes: Vec<String>,
        #[serde(default)]
        bars: Vec<String>,
        #[serde(default)]
        orderbooks: Vec<String>,
        #[serde(default)]
        updated_bars: Vec<String>,
        #[serde(default)]
        daily_bars: Vec<String>,
        #[serde(default)]
        statuses: Vec<String>,
        #[serde(default)]
        lulds: Vec<String>,
        #[serde(default)]
        corrections: Vec<String>,
        #[serde(default)]
        cancel_errors: Vec<String>,
    },

    #[serde(rename = "b")]
    Bar(Bar),

    #[serde(rename = "q")]
    Quote(Quote),

    #[serde(rename = "t")]
    Trade(Trade),
}

impl Message for AlpacaMessage {}

#[derive(Debug, Deserialize, Clone)]
pub struct Bar {
    #[serde(rename = "S")]
    pub symbol: String,

    #[serde(rename = "o")]
    pub open: f64,

    #[serde(rename = "h")]
    pub high: f64,

    #[serde(rename = "l")]
    pub low: f64,

    #[serde(rename = "c")]
    pub close: f64,

    #[serde(rename = "v")]
    pub volume: f64,

    #[serde(rename = "t")]
    pub timestamp: String,

    #[serde(rename = "n")]
    pub trade_count: Option<u64>,

    #[serde(rename = "vw")]
    pub vwap: Option<f64>,
}

impl Bar {
    pub fn price_change(&self) -> f64 {
        self.close - self.open
    }

    pub fn price_change_percent(&self) -> f64 {
        (self.price_change() / self.open) * 100.0
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Quote {
    #[serde(rename = "S")]
    pub symbol: String,

    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,

    #[serde(rename = "bp")]
    pub bid_price: f64,

    #[serde(rename = "bs")]
    pub bid_size: f64,

    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,

    #[serde(rename = "ap")]
    pub ask_price: f64,

    #[serde(rename = "as")]
    pub ask_size: f64,

    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,

    #[serde(rename = "z")]
    pub tape: Option<String>,

    #[serde(rename = "t")]
    pub timestamp: String,
}

impl Quote {
    pub fn spread(&self) -> f64 {
        self.ask_price - self.bid_price
    }

    pub fn spread_bps(&self) -> f64 {
        (self.spread() / self.bid_price) * 10000.0
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Trade {
    #[serde(rename = "T")]
    pub t: Option<String>,

    #[serde(rename = "S")]
    pub symbol: String,

    #[serde(rename = "i")]
    pub id: u64,

    #[serde(rename = "x")]
    pub exchange: Option<String>,

    #[serde(rename = "p")]
    pub price: f64,

    #[serde(rename = "s")]
    pub size: f64,

    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,

    #[serde(rename = "z")]
    pub tape: Option<String>,

    #[serde(rename = "tks")]
    pub tks: Option<String>,

    #[serde(rename = "t")]
    pub timestamp: String,
}
