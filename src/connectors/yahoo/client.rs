use super::types::{IncomeStatementRow, YahooMessage};
use crate::{
    connectors::yahoo::types::{BalanceSheetRow, CashflowRow},
    core::{MessageBatch, MessageSource},
};
use std::pin::Pin;
use tokio::time::{Duration, sleep};
use tracing::{debug, error};
use yfinance_rs::{Ticker, YfClient, YfError};
pub struct YahooClient {
    client: YfClient,
    symbols: Vec<String>,
    timeout_ms: u64,
}

impl MessageSource<YahooMessage> for YahooClient {
    fn run<'a>(
        &'a mut self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Now fetch income statements for each symbol
            for symbol in &self.symbols {
                if let Err(err) = self
                    .fetch_ticker_quarterly_income_stmt(symbol, tx.clone())
                    .await
                {
                    error!("Failed to fetch income statement for {}: {}", symbol, err);
                }
                sleep(Duration::from_millis(self.timeout_ms)).await;
                if let Err(err) = self
                    .fetch_ticker_quarterly_balance_sheet(symbol, tx.clone())
                    .await
                {
                    error!("Failed to fetch income statement for {}: {}", symbol, err);
                }
                sleep(Duration::from_millis(self.timeout_ms)).await;
                if let Err(err) = self
                    .fetch_ticker_quarterly_cashflow(symbol, tx.clone())
                    .await
                {
                    error!("Failed to fetch income statement for {}: {}", symbol, err);
                }
                // Throttle requests to avoid rate limiting
                sleep(Duration::from_millis(self.timeout_ms)).await;
            }

            Ok(())
        })
    }
}

impl YahooClient {
    pub fn new(symbols: Vec<String>, timeout_ms: u64) -> Self {
        Self {
            client: YfClient::default(),
            symbols,
            timeout_ms,
        }
    }

    pub async fn fetch_ticker_quarterly_income_stmt(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> Result<(), YfError> {
        debug!("Fetching income data for {symbol}");
        let ticker = Ticker::new(&self.client, symbol);
        match ticker.quarterly_income_stmt(None).await {
            Ok(stmt) => {
                let messages: Vec<YahooMessage> = stmt
                    .into_iter()
                    .map(|row| {
                        YahooMessage::IncomeStatement(IncomeStatementRow {
                            symbol: (symbol.to_string()),
                            inner: (row),
                        })
                    })
                    .collect();
                // debug!(messages);
                let _ = tx.send(messages).await;
            }
            Err(err) => {
                error!("error! {err}");
            }
        };
        Ok(())
    }

    pub async fn fetch_ticker_quarterly_balance_sheet(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> Result<(), YfError> {
        debug!("Fetching balance sheet data for {symbol}");
        let ticker = Ticker::new(&self.client, symbol);
        match ticker.quarterly_balance_sheet(None).await {
            Ok(stmt) => {
                let messages: Vec<YahooMessage> = stmt
                    .into_iter()
                    .map(|row| {
                        YahooMessage::BalanceSheet(BalanceSheetRow {
                            symbol: (symbol.to_string()),
                            inner: (row),
                        })
                    })
                    .collect();
                // debug!(messages);
                let _ = tx.send(messages).await;
            }
            Err(err) => {
                error!("error! {err}");
            }
        };
        Ok(())
    }

    pub async fn fetch_ticker_quarterly_cashflow(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> Result<(), YfError> {
        debug!("Fetching cashflow data for {symbol}");
        let ticker = Ticker::new(&self.client, symbol);
        match ticker.quarterly_cashflow(None).await {
            Ok(stmt) => {
                let messages: Vec<YahooMessage> = stmt
                    .into_iter()
                    .map(|row| {
                        YahooMessage::Cashflow(CashflowRow {
                            symbol: (symbol.to_string()),
                            inner: (row),
                        })
                    })
                    .collect();
                // debug!(messages);
                let _ = tx.send(messages).await;
            }
            Err(err) => {
                error!("error! {err}");
            }
        };
        Ok(())
    }
}
