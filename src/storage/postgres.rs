//! PostgreSQL-backed message sink for Alpaca market data.

use std::future::Future;
use std::pin::Pin;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime};
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};

use crate::core::{MessageBatch, MessageSink};

use crate::connectors::alpaca::types::{AlpacaMessage, Bar, Quote, Trade};

/// Thin Tokio Postgres wrapper that stores Alpaca messages.
pub struct Database {
    client: Client,
}

impl MessageSink<AlpacaMessage> for Database {
    fn name(&self) -> &'static str {
        "postgres"
    }

    fn handle_batch<'a>(
        &'a self,
        batch: MessageBatch<AlpacaMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut bars = Vec::new();
            let mut quotes = Vec::new();
            let mut trades = Vec::new();

            for message in batch {
                match message {
                    AlpacaMessage::Bar(bar) => bars.push(bar),
                    AlpacaMessage::Quote(quote) => quotes.push(quote),
                    AlpacaMessage::Trade(trade) => trades.push(trade),
                    _ => {}
                }
            }

            if !bars.is_empty() {
                self.insert_bars_batch(&bars).await?;
            }

            if !quotes.is_empty() {
                self.insert_quotes_batch(&quotes).await?;
            }

            if !trades.is_empty() {
                self.insert_trades_batch(&trades).await?;
            }

            Ok(())
        })
    }
}

impl Database {
    /// Inserts bar records individually, skipping conflicts.
    async fn insert_bars_batch(&self, bars: &[Bar]) -> Result<()> {
        for bar in bars {
            let timestamp = parse_timestamp(&bar.timestamp)?;
            let trade_count = bar.trade_count.map(|count| count as i64);
            let vwap = bar.vwap;

            self.insert_bar(
                &bar.symbol,
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                bar.volume as i64,
                timestamp,
                &trade_count,
                &vwap,
            )
            .await?;
        }

        Ok(())
    }

    /// Inserts quote records individually, keeping the input order.
    async fn insert_quotes_batch(&self, quotes: &[Quote]) -> Result<()> {
        for quote in quotes {
            let timestamp = parse_timestamp(&quote.timestamp)?;
            let bid_size = quote.bid_size as i64;
            let ask_size = quote.ask_size as i64;

            self.insert_quote(
                &quote.symbol,
                &quote.bid_exchange,
                quote.bid_price,
                bid_size,
                &quote.ask_exchange,
                quote.ask_price,
                ask_size,
                timestamp,
                &quote.tape,
            )
            .await?;
        }

        Ok(())
    }

    /// Inserts trade records individually, ignoring duplicates by ID.
    async fn insert_trades_batch(&self, trades: &[Trade]) -> Result<()> {
        for trade in trades {
            let timestamp = parse_timestamp(&trade.timestamp)?;

            self.insert_trade(
                trade.id as i64,
                &trade.symbol,
                &trade.exchange,
                trade.price,
                trade.size as i64,
                timestamp,
                &trade.tape,
                &trade.tks,
            )
            .await?;
        }

        Ok(())
    }

    /// Connects to Postgres and spawns the connection task.
    pub async fn connect(connection_string: &str) -> Result<Self, tokio_postgres::Error> {
        info!("Connecting to database...");

        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Database connection error: {}", e);
            }
        });

        info!("Database connected successfully...");

        Ok(Database { client })
    }

    /// Creates the tables required for storing market data if they are missing.
    pub async fn initialize_schema(&self) -> Result<(), tokio_postgres::Error> {
        info!("Initializing database schema...");

        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS bars (
                    id SERIAL PRIMARY KEY,
                    symbol VARCHAR(10) NOT NULL,
                    open DOUBLE PRECISION NOT NULL,
                    high DOUBLE PRECISION NOT NULL,
                    low DOUBLE PRECISION NOT NULL,
                    close DOUBLE PRECISION NOT NULL,
                    volume BIGINT NOT NULL,
                    timestamp TIMESTAMP NOT NULL,
                    trade_count BIGINT,
                    vwap DOUBLE PRECISION,
                    received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(symbol, timestamp)
                )",
                &[],
            )
            .await?;

        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS quotes (
                    id SERIAL PRIMARY KEY,
                    symbol VARCHAR(10) NOT NULL,
                    bid_exchange VARCHAR(10),
                    bid_price DOUBLE PRECISION NOT NULL,
                    bid_size BIGINT NOT NULL,
                    ask_exchange VARCHAR(10),
                    ask_price DOUBLE PRECISION NOT NULL,
                    ask_size BIGINT NOT NULL,
                    timestamp TIMESTAMP NOT NULL,
                    tape VARCHAR(5),
                    received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                &[],
            )
            .await?;

        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS trades (
                    id SERIAL PRIMARY KEY,
                    trade_id BIGINT NOT NULL,
                    symbol VARCHAR(10) NOT NULL,
                    exchange VARCHAR(10),
                    price DOUBLE PRECISION NOT NULL,
                    size BIGINT NOT NULL,
                    timestamp TIMESTAMP NOT NULL,
                    tape VARCHAR(5),
                    tks VARCHAR(5),
                    received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(trade_id, symbol)
                )",
                &[],
            )
            .await?;

        info!("Database schema initialized");
        Ok(())
    }

    /// Inserts a single bar row, ignoring existing symbol/timestamp combinations.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_bar(
        &self,
        symbol: &str,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
        timestamp: NaiveDateTime,
        trade_count: &Option<i64>,
        vwap: &Option<f64>,
    ) -> Result<(), tokio_postgres::Error> {
        self.client
            .execute(
                "INSERT INTO bars (symbol, open, high, low, close, volume, timestamp, trade_count, vwap)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (symbol, timestamp) DO NOTHING",
                &[&symbol, &open, &high, &low, &close, &volume, &timestamp, &trade_count, &vwap],
            )
            .await?;

        Ok(())
    }

    /// Inserts a single quote row.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_quote(
        &self,
        symbol: &str,
        bid_exchange: &Option<String>,
        bid_price: f64,
        bid_size: i64,
        ask_exchange: &Option<String>,
        ask_price: f64,
        ask_size: i64,
        timestamp: NaiveDateTime,
        tape: &Option<String>,
    ) -> Result<(), tokio_postgres::Error> {
        self.client
            .execute(
                "INSERT INTO quotes (symbol, bid_exchange, bid_price, bid_size, 
                                    ask_exchange, ask_price, ask_size, timestamp, tape)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &symbol,
                    &bid_exchange,
                    &bid_price,
                    &bid_size,
                    &ask_exchange,
                    &ask_price,
                    &ask_size,
                    &timestamp,
                    &tape,
                ],
            )
            .await?;

        Ok(())
    }

    /// Inserts a single trade row, de-duplicating by trade ID and symbol.
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_trade(
        &self,
        trade_id: i64,
        symbol: &str,
        exchange: &Option<String>,
        price: f64,
        size: i64,
        timestamp: NaiveDateTime,
        tape: &Option<String>,
        tks: &Option<String>,
    ) -> Result<(), tokio_postgres::Error> {
        self.client
            .execute(
                "INSERT INTO trades (trade_id, symbol, exchange, price, size, timestamp, tape, tks)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 ON CONFLICT (trade_id, symbol) DO NOTHING",
                &[
                    &trade_id, &symbol, &exchange, &price, &size, &timestamp, &tape, &tks,
                ],
            )
            .await?;

        Ok(())
    }
}

fn parse_timestamp(value: &str) -> Result<NaiveDateTime> {
    Ok(DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("failed to parse RFC3339 timestamp: {value}"))?
        .naive_utc())
}
