use serde::Deserialize;

// This derives the Debug and Deserialize traits for AlpacaMessage, which allows for formatted output
// via {:?} and deserialization from formats like JSON, respectively.
//
// #[serde(tag = "T", rename_all = "lowercase")] tells Serde to use an
// externally tagged enum design, using the "T" field in the incoming map (e.g., JSON).
// The value of "T" selects the enum variant, matching lowercase variant names unless explicitly renamed.
// For example, a JSON object with "T": "b" will deserialize into the Bar variant.
#[derive(Debug, Deserialize)]
#[serde(tag = "T", rename_all = "lowercase")]
pub enum AlpacaMessage {
    #[serde(rename = "success")]
    Success { 
        msg: String 
    },
    #[serde(rename = "error")]
    Error { 
        code: u16,
        msg: String 
    },
    #[serde(rename = "subscription")]
    #[serde(rename_all = "camelCase")]
    Subscription { 
        trades: Vec<String>,
        quotes: Vec<String>,
        bars: Vec<String>,
        updated_bars: Vec<String>,
        daily_bars: Vec<String>,
        statuses: Vec<String>,
        lulds: Vec<String>,
        corrections: Vec<String>,
        cancel_errors: Vec<String>,
    },
    #[serde(rename = "b")]
    Bar(Bar),
    #[serde(rename = "q")]
    Quote(Quote),
    #[serde(rename = "t")]
    Trade(Trade),
}

#[derive(Debug, Deserialize)]
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
    pub volume: u64,
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "n")]
    pub trade_count: u64,
    #[serde(rename = "vw")]
    pub volume_weighted_average_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "bx")]
    pub bid_exchange: String,
    #[serde(rename = "bp")]
    pub bid_price: f64,
    #[serde(rename = "bs")]
    pub bid_size: u64,
    #[serde(rename = "ax")]
    pub ask_exchange: String,
    #[serde(rename = "ap")]
    pub ask_price: f64,
    #[serde(rename = "as")]
    pub ask_size: u64,
    #[serde(rename = "c")]
    pub conditions: Vec<String>,
    #[serde(rename = "z")]
    pub tape: String,
    #[serde(rename = "t")]
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct Trade {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub id: u64,
    #[serde(rename = "x")]
    pub exchange: String,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: u64,
    #[serde(rename = "c")]
    pub conditions: Vec<String>,
    #[serde(rename = "z")]
    pub tape: String,
    #[serde(rename = "t")]
    pub timestamp: String,
}