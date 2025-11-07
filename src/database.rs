// Leanring 

use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls}; // NoTls disables TLS encryption for local/dev use
use tracing::info;



pub struct Database {
    client: Client,
}

impl Database {

    pub async fn connect(connection_string: &str) -> Result<Self> {
        info!("Connecting to database...");

        let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
            .await
            // If the async connection fails, attach extra context for clearer error reporting.
            .context("Failed to connect to database")?;

        
        // this is required for tokio_postgres to work
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        info!("Database connected successfully...");

        Ok(Database { client })
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema...");
        
        // Create bars table for OHLCV data
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
                    trade_count BIGINT NOT NULL,
                    vwap DOUBLE PRECISION NOT NULL,
                    received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(symbol, timestamp)
                )",
                &[],
            )
            .await
            .context("Failed to create bars table")?;
        
        // Create quotes table for bid/ask data
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS quotes (
                    id SERIAL PRIMARY KEY,
                    symbol VARCHAR(10) NOT NULL,
                    bid_exchange VARCHAR(10) NOT NULL,
                    bid_price DOUBLE PRECISION NOT NULL,
                    bid_size BIGINT NOT NULL,
                    ask_exchange VARCHAR(10) NOT NULL,
                    ask_price DOUBLE PRECISION NOT NULL,
                    ask_size BIGINT NOT NULL,
                    timestamp TIMESTAMP NOT NULL,
                    tape VARCHAR(5) NOT NULL,
                    received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )",
                &[],
            )
            .await
            .context("Failed to create quotes table")?;
        
        // Create trades table
        self.client
            .execute(
                "CREATE TABLE IF NOT EXISTS trades (
                    id SERIAL PRIMARY KEY,
                    trade_id BIGINT NOT NULL,
                    symbol VARCHAR(10) NOT NULL,
                    exchange VARCHAR(10) NOT NULL,
                    price DOUBLE PRECISION NOT NULL,
                    size BIGINT NOT NULL,
                    timestamp TIMESTAMP NOT NULL,
                    tape VARCHAR(5) NOT NULL,
                    received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(trade_id, symbol)
                )",
                &[],
            )
            .await
            .context("Failed to create trades table")?;
        
        info!("Database schema initialized");
        Ok(())
    }

        /// Insert a bar (OHLCV) record
    /// 
    /// Learning: PARAMETERIZED QUERIES
    /// The $1, $2, etc. are placeholders for parameters
    /// This prevents SQL injection and handles type conversion
    pub async fn insert_bar(
        &self,
        symbol: &str,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
        timestamp: &str,
        trade_count: i64,
        vwap: f64,
    ) -> Result<()> {
        self.client
            .execute(
                "INSERT INTO bars (symbol, open, high, low, close, volume, timestamp, trade_count, vwap)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (symbol, timestamp) DO NOTHING",
                &[&symbol, &open, &high, &low, &close, &volume, &timestamp, &trade_count, &vwap],
            )
            .await
            .context("Failed to insert bar")?;
        
        Ok(())
    }

    /// Insert a quote record
    pub async fn insert_quote(
        &self,
        symbol: &str,
        bid_exchange: &str,
        bid_price: f64,
        bid_size: i64,
        ask_exchange: &str,
        ask_price: f64,
        ask_size: i64,
        timestamp: &str,
        tape: &str,
    ) -> Result<()> {
        self.client
            .execute(
                "INSERT INTO quotes (symbol, bid_exchange, bid_price, bid_size, 
                                    ask_exchange, ask_price, ask_size, timestamp, tape)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[&symbol, &bid_exchange, &bid_price, &bid_size, 
                    &ask_exchange, &ask_price, &ask_size, &timestamp, &tape],
            )
            .await
            .context("Failed to insert quote")?;
        
        Ok(())
    }
    /// Insert a trade record
    pub async fn insert_trade(
        &self,
        trade_id: i64,
        symbol: &str,
        exchange: &str,
        price: f64,
        size: i64,
        timestamp: &str,
        tape: &str,
    ) -> Result<()> {
        self.client
            .execute(
                "INSERT INTO trades (trade_id, symbol, exchange, price, size, timestamp, tape)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (trade_id, symbol) DO NOTHING",
                &[&trade_id, &symbol, &exchange, &price, &size, &timestamp, &tape],
            )
            .await
            .context("Failed to insert trade")?;
        
        Ok(())
    }

}