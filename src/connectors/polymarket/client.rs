//! Polymarket CLOB client for fetching market data.

use std::future::Future;
use std::pin::Pin;

use anyhow::{Context, Result};
use polymarket_rs_client::ClobClient;
use tokio::time::{Duration, sleep};
use tracing::{debug, info, warn};

use super::types::{Market, PolymarketMessage};
use crate::core::{MessageBatch, MessageSource};

/// Polymarket API host
const HOST: &str = "https://clob.polymarket.com";

/// Polygon chain ID
const POLYGON: u64 = 137;

/// Client for fetching Polymarket data.
pub struct PolymarketClient {
    /// Private key for authentication
    private_key: String,
    /// Delay between API requests in milliseconds
    request_delay_ms: u64,
}

impl PolymarketClient {
    /// Create a new PolymarketClient.
    ///
    /// # Arguments
    /// * `private_key` - Polymarket private key for API authentication
    /// * `request_delay_ms` - Delay between paginated API requests in milliseconds
    pub fn new(private_key: String, request_delay_ms: u64) -> Self {
        Self {
            private_key,
            request_delay_ms,
        }
    }

    /// Fetch all markets with pagination and send them through the channel.
    async fn fetch_all_markets(
        &self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<PolymarketMessage>>,
    ) -> Result<()> {
        // Initialize the CLOB client
        let mut client = ClobClient::with_l1_headers(HOST, &self.private_key, POLYGON);

        // Create or derive API keys
        let keys = client
            .create_or_derive_api_key(None)
            .await
            .context("Failed to create or derive API key")?;
        client.set_api_creds(keys);

        let mut next_cursor: Option<String> = None;
        let mut page_count = 0;
        let mut total_markets = 0;

        loop {
            page_count += 1;
            debug!(
                page = page_count,
                cursor = ?next_cursor,
                "Fetching markets page"
            );

            let response = client
                .get_markets(next_cursor.as_deref())
                .await
                .context("Failed to fetch markets")?;

            // Extract data array from response
            if let Some(data) = response.get("data").and_then(|d| d.as_array()) {
                let page_markets_count = data.len();
                info!(
                    page = page_count,
                    markets = page_markets_count,
                    "Received markets page"
                );

                // Parse markets and send as messages
                let messages: Vec<PolymarketMessage> = data
                    .iter()
                    .filter_map(|market_json| {
                        match serde_json::from_value::<Market>(market_json.clone()) {
                            Ok(market) => Some(PolymarketMessage::Market(market)),
                            Err(e) => {
                                warn!(
                                    error = %e,
                                    "Failed to parse market, skipping"
                                );
                                None
                            }
                        }
                    })
                    .collect();

                if !messages.is_empty() {
                    tx.send(messages)
                        .await
                        .context("Failed to send market messages")?;
                }

                total_markets += page_markets_count;
            }

            // Check if there's a next page
            if let Some(cursor) = response.get("next_cursor").and_then(|c| c.as_str()) {
                if cursor.is_empty() {
                    info!(
                        total_markets = total_markets,
                        pages = page_count,
                        "Finished fetching all markets"
                    );
                    break;
                }
                next_cursor = Some(cursor.to_string());
            } else {
                info!(
                    total_markets = total_markets,
                    pages = page_count,
                    "Finished fetching all markets"
                );
                break;
            }

            // Rate limiting delay
            sleep(Duration::from_millis(self.request_delay_ms)).await;
        }

        Ok(())
    }
}

impl MessageSource<PolymarketMessage> for PolymarketClient {
    fn run<'a>(
        &'a mut self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<PolymarketMessage>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { self.fetch_all_markets(tx).await })
    }
}
