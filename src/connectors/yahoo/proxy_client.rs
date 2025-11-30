use super::types::{IncomeStatementRow, YahooMessage};
use crate::{
    connectors::yahoo::types::{BalanceSheetRow, CalendarDateType, CalendarEntry, CashflowRow},
    core::{MessageBatch, MessageSource},
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{Duration, sleep};
use tracing::debug;
use yfinance_rs::{Ticker, YfClient};

pub struct ProxyYahooClient {
    clients: Vec<YfClient>,
    counter: Arc<AtomicUsize>,
    symbols: Vec<String>,
    timeout_ms: u64,
}

impl MessageSource<YahooMessage> for ProxyYahooClient {
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

impl ProxyYahooClient {
    pub fn new(
        proxies: Vec<String>,
        symbols: Vec<String>,
        timeout_ms: u64,
    ) -> anyhow::Result<Self> {
        let mut clients = Vec::with_capacity(proxies.len() + 1);

        // Create default client without proxy
        clients.push(YfClient::default());

        // Create clients with proxies
        for proxy in proxies {
            let client = YfClient::builder().proxy(&proxy).build().map_err(|e| {
                anyhow::anyhow!("Failed to create client with proxy {}: {}", proxy, e)
            })?;
            clients.push(client);
        }

        Ok(Self {
            clients,
            counter: Arc::new(AtomicUsize::new(0)),
            symbols,
            timeout_ms,
        })
    }

    fn get_next_client(&self) -> &YfClient {
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        &self.clients[idx]
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
        sleep(Duration::from_millis(self.timeout_ms)).await;

        self.fetch_calendars(symbol, tx.clone()).await?;
        sleep(Duration::from_millis(self.timeout_ms)).await;
        Ok(())
    }

    async fn fetch_income_statement(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        debug!("Fetching income statement data for {symbol}");
        let client = self.get_next_client();
        let ticker = Ticker::new(client, symbol);
        let stmt = ticker
            .quarterly_income_stmt(None)
            .await
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

        tx.send(messages)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send income statement messages: {e}"))?;
        Ok(())
    }

    async fn fetch_balance_sheet(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        debug!("Fetching balance sheet data for {symbol}");
        let client = self.get_next_client();
        let ticker = Ticker::new(client, symbol);
        let stmt = ticker
            .quarterly_balance_sheet(None)
            .await
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

        tx.send(messages)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send balance sheet messages: {e}"))?;
        Ok(())
    }

    async fn fetch_cashflow(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        debug!("Fetching cashflow data for {symbol}");
        let client = self.get_next_client();
        let ticker = Ticker::new(client, symbol);
        let stmt = ticker
            .quarterly_cashflow(None)
            .await
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

        tx.send(messages)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send cashflow messages: {e}"))?;
        Ok(())
    }

    async fn fetch_calendars(
        &self,
        symbol: &str,
        tx: tokio::sync::mpsc::Sender<MessageBatch<YahooMessage>>,
    ) -> anyhow::Result<()> {
        debug!("Fetching cashflow data for {symbol}");
        let client = self.get_next_client();
        let ticker = Ticker::new(client, symbol);
        let calendars = ticker
            .calendar()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch cashflow for {symbol}: {e}"))?;

        let earnings: Vec<YahooMessage> = calendars
            .earnings_dates
            .into_iter()
            .map(|row| {
                YahooMessage::Calendar(CalendarEntry {
                    symbol: symbol.to_string(),
                    date_type: CalendarDateType::Earnings,
                    date: row.naive_utc(),
                })
            })
            .collect();

        let dividend_payment_dates: Vec<YahooMessage> = calendars
            .dividend_payment_date
            .into_iter()
            .map(|row| {
                YahooMessage::Calendar(CalendarEntry {
                    symbol: symbol.to_string(),
                    date_type: CalendarDateType::DividendPayment,
                    date: row.naive_utc(),
                })
            })
            .collect();

        let ex_dividend_dates: Vec<YahooMessage> = calendars
            .ex_dividend_date
            .into_iter()
            .map(|row| {
                YahooMessage::Calendar(CalendarEntry {
                    symbol: symbol.to_string(),
                    date_type: CalendarDateType::ExDividend,
                    date: row.naive_utc(),
                })
            })
            .collect();

        let mut messages: Vec<YahooMessage> = Vec::new();
        messages.extend(earnings);
        messages.extend(dividend_payment_dates);
        messages.extend(ex_dividend_dates);

        tx.send(messages)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send calendar messages: {e}"))?;
        Ok(())
    }
}
