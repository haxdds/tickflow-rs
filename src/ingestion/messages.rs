use serde::Deserialize;

/*
  This block derives the Debug and Deserialize traits for AlpacaMessage, which enables formatted output
  using {:?} and deserialization from formats such as JSON.

  The attribute #[serde(tag = "T", rename_all = "lowercase")] configures Serde to use externally tagged
  enum representation, keyed by the "T" field in the input map (such as JSON). The value of "T" chooses
  which enum variant to use, matching the variant's name in lowercase (unless specifically renamed).

  For instance, if a JSON object has "T": "b", it will be deserialized into the Bar variant.
*/
#[derive(Debug, Deserialize)]
#[serde(tag = "T", rename_all = "lowercase")]
pub enum AlpacaMessage {
    // The #[serde(rename = "success")] attribute is used here for explicitness or if you want to ensure this variant always maps to the string "success" in the input data.
    // 
    // The #[serde(rename_all = "lowercase")] at the enum level means that, by default, variant names will be lowercased unless a variant-specific #[serde(rename = ...)] is given.
    // If you remove #[serde(rename = "success")], Serde will still map the Success variant to "success" in lowercase because of the rename_all.
    // 
    // So, in this case, you do not strictly need the #[serde(rename = "success")]; it's redundant, but doesn't hurt unless you want to override or clarify.
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
    pub vwap: f64,
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

impl AlpacaMessage {
    pub fn parse(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}