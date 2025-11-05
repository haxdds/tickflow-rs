use serde::Serialize;


// Configuration for WebSocket consumer
pub struct IngestionConfig {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
    pub symbols: Vec<String>
}

// This is an implementation block for the `IngestionConfig` struct.
// The `impl` keyword is used to define associated functions or methods for a type.
// Here, we define an associated function named `new` that constructs a new instance of `IngestionConfig`.
// It takes three parameters: `api_key`, `api_secret`, and a vector of `symbols`, all as owned (not borrowed) values.
//
// The function returns `Self`, which is an alias for `IngestionConfig` within this context.
//
// Inside `new`, we use "struct update syntax" (a struct literal) to instantiate `IngestionConfig`.
// - The fields `api_key`, `api_secret`, and `symbols` are set from the arguments of the same name using field-init shorthand
// - The `base_url` is hard-coded as a specific websocket endpoint string, converted to a `String` with `.to_string()`
//
// By using `Self` in this way, the code is concise and clear. This is a common Rust pattern for builder/constructor methods.

impl IngestionConfig {
    pub fn new(api_key: String, api_secret: String, symbols: Vec<String>) -> Self {
        Self {
            api_key, // Sets the struct field `api_key` to the argument `api_key`
            api_secret, // Sets the struct field `api_secret` to the argument `api_secret`
            base_url: "wss://stream.data.alpaca.markets/v2/iex".to_string(), // Hardcoded websocket base URL
            symbols, // Sets the struct field `symbols` to the argument `symbols`
        }
    }
}

/// Authentication message
#[derive(Serialize)]
pub struct AuthMessage {
    pub action: String,
    pub key: String,
    pub secret: String,
}

/// Subscription message
#[derive(Serialize)]
pub struct SubscribeMessage {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<String>>,
}
