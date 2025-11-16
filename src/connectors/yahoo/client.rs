use yfinance_rs::{Ticker, YfClient, YfError};
// use yfinance_rs::fundamentals::{BalanceSheetRow, IncomeStatementRow};
use crate::core::{MessageSource, MessageBatch};
use super::types::{YahooMessage};
use tracing::{error};
use std::pin::Pin;
use super::symbols::load_symbols;
use tokio::time::{sleep, Duration};
pub struct YahooClient {
    client: YfClient,
}

impl MessageSource<YahooMessage> for YahooClient {
    fn run<'a>(
        &'a mut self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Load all symbols into memory first
            let symbols = load_symbols().await?;

            // Now fetch income statements for each symbol
            for symbol in symbols {
                if let Err(err) = self.fetch_ticker_quarterly_income(&symbol, tx.clone()).await {
                    error!("Failed to fetch income statement for {}: {}", symbol, err);
                }
                
                // Throttle requests to avoid rate limiting
                sleep(Duration::from_millis(1000)).await;
            }
            
            Ok(())
        })
    }
}

impl YahooClient {
    pub fn new() -> Self {
        Self {
            client: YfClient::default()
        }
    }

    pub async fn fetch_ticker_quarterly_income(&mut self, ticker: &str, tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>) -> Result<(), YfError> {
        let ticker = Ticker::new(&self.client, ticker);
        match ticker.quarterly_income_stmt(None).await {
            Ok(stmt) => {
                let messages: Vec<YahooMessage> = stmt
                    .into_iter()
                    .map(|row| YahooMessage::IncomeStatement(row))
                    .collect();
                let _ = tx.send(messages).await;
            }
            Err(err) => {
                error!("error! {err}");
            }
        };
        Ok(())
    }
}


