use super::types::{IncomeStatementRow, YahooMessage};
use crate::{
    connectors::yahoo::types::{BalanceSheetRow, CashflowRow},
    core::{MessageBatch, MessageSource},
};
use std::pin::Pin;
use tokio::time::{Duration, sleep};
use tracing::{debug};
use yfinance_rs::{Ticker, YfClient};

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
            for symbol in &self.symbols {
                self.fetch_all_statements(symbol, tx.clone()).await?;
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

    async fn fetch_all_statements(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        self.fetch_income_statement(symbol, tx.clone()).await?;
        sleep(Duration::from_millis(self.timeout_ms)).await;
        
        self.fetch_balance_sheet(symbol, tx.clone()).await?;
        sleep(Duration::from_millis(self.timeout_ms)).await;
        
        self.fetch_cashflow(symbol, tx.clone()).await?;
        Ok(())
    }

    async fn fetch_income_statement(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        debug!("Fetching income statement data for {symbol}");
        let ticker = Ticker::new(&self.client, symbol);
        let stmt = ticker.quarterly_income_stmt(None).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch income statement for {symbol}: {e}"))?;
        
        let messages: Vec<YahooMessage> = stmt
            .into_iter()
            .map(|row| {
                YahooMessage::IncomeStatement(IncomeStatementRow {
                    symbol: symbol.to_string(),
                    inner: row,
                })
            })
            .collect();
        
        tx.send(messages).await
            .map_err(|e| anyhow::anyhow!("Failed to send income statement messages: {e}"))?;
        Ok(())
    }

    async fn fetch_balance_sheet(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        debug!("Fetching balance sheet data for {symbol}");
        let ticker = Ticker::new(&self.client, symbol);
        let stmt = ticker.quarterly_balance_sheet(None).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch balance sheet for {symbol}: {e}"))?;
        
        let messages: Vec<YahooMessage> = stmt
            .into_iter()
            .map(|row| {
                YahooMessage::BalanceSheet(BalanceSheetRow {
                    symbol: symbol.to_string(),
                    inner: row,
                })
            })
            .collect();
        
        tx.send(messages).await
            .map_err(|e| anyhow::anyhow!("Failed to send balance sheet messages: {e}"))?;
        Ok(())
    }

    async fn fetch_cashflow(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        debug!("Fetching cashflow data for {symbol}");
        let ticker = Ticker::new(&self.client, symbol);
        let stmt = ticker.quarterly_cashflow(None).await
            .map_err(|e| anyhow::anyhow!("Failed to fetch cashflow for {symbol}: {e}"))?;
        
        let messages: Vec<YahooMessage> = stmt
            .into_iter()
            .map(|row| {
                YahooMessage::Cashflow(CashflowRow {
                    symbol: symbol.to_string(),
                    inner: row,
                })
            })
            .collect();
        
        tx.send(messages).await
            .map_err(|e| anyhow::anyhow!("Failed to send cashflow messages: {e}"))?;
        Ok(())
    }
}
