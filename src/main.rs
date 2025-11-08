// Learning Goals:
// - Tokio runtime and async/await
// - Environment variables with dotenv
// - Structured logging with tracing
// - Channel creation and usage
// - Concurrent tasks with tokio::spawn
// - Error handling with Result and anyhow


use anyhow::{Context, Result};
use dotenvy::dotenv;
use std::env;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber;

use tickflow::database::Database;
use tickflow::websocket::AlpacaWebSocketClient;
use tickflow::prcoessor::MessageProcessor;
// use crate::{AlpacaWebSocketClient, Database, MessageProcessor};


/// Main entry point
/// 
/// Learning: TOKIO RUNTIME
/// #[tokio::main] transforms the async main into a synchronous one
/// It creates a Tokio runtime and runs the async code on it
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    // Learning: TRACING
    // This sets up structured logging for the application
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("🚀 Starting Alpaca Market Data Pipeline");
    
    // Load environment variables from .env file
    // Learning: ENVIRONMENT CONFIGURATION
    // This is a common pattern for managing configuration
    dotenv().ok(); // .ok() converts Result to Option, ignoring errors
    
    // Get configuration from environment
    let db_url = env::var("DATABASE_URL")
        .context("DATABASE_URL must be set")?;
    
    let alpaca_api_key = env::var("APCA_API_KEY_ID")
        .context("APCA_API_KEY_ID must be set")?;
    
    let alpaca_api_secret = env::var("APCA_API_SECRET_KEY")
        .context("APCA_API_SECRET_KEY must be set")?;
    
    let alpaca_ws_url = env::var("ALPACA_WS_URL")
        .unwrap_or_else(|_| "wss://stream.data.alpaca.markets/v2/iex".to_string());
    
    println!(
        "db_url: {}, alpaca_api_key: {}, alpaca_api_secret: {}, alpaca_ws_url: {}",
        db_url, alpaca_api_key, alpaca_api_secret, alpaca_ws_url
    );
    // Connect to database
    info!("📊 Connecting to database...");
    let database = Database::connect(&db_url).await?;
    
    // Initialize schema
    database.initialize_schema().await?;
    
    // Create channel for passing messages
    // Learning: MPSC CHANNEL CREATION
    // mpsc::channel(buffer_size) creates a bounded channel
    // buffer_size determines how many messages can be queued
    // If full, send() will wait until space is available
    let (tx, rx) = mpsc::channel(1000);
    
    info!("📡 Channel created with buffer size 1000");
    
    // Create WebSocket client
    let mut ws_client = AlpacaWebSocketClient::new(
        &alpaca_ws_url,
        &alpaca_api_key,
        &alpaca_api_secret,
    );
    
    // Create message processor
    let processor = MessageProcessor::new(database);
    
    // Spawn WebSocket task
    // Learning: TOKIO::SPAWN
    // This creates a new async task that runs concurrently
    // It runs independently on the tokio runtime
    let ws_handle = {
        let tx = tx.clone(); // Clone the sender for the WebSocket task
        tokio::spawn(async move {
            if let Err(e) = ws_client.connect().await {
                error!("WebSocket error: {}", e);
            }
            if let Err(e) = ws_client.authenticate().await {
                error!("Authenticate error: {}", e);
            }
            if let Err(e) = ws_client.subscribe(&[], &["ETH/USD"], &[]).await {
                error!("Subscribe error: {}", e);
            }

            if let Err(e) = ws_client.stream_messages(tx).await {
                error!("Stream error: {}", e);
            }
        })
    };
    
    // // Spawn processor task
    let processor_handle = tokio::spawn(async move {
        if let Err(e) = processor.process_messages(rx).await {
            error!("Processor error: {}", e);
        }

    });
    
    info!("✅ Pipeline started");
    // info!("   - WebSocket: receiving market data");
    // info!("   - Processor: storing to database");
    // info!("   - Press Ctrl+C to stop");
    
    // Wait for both tasks to complete
    // Learning: JOINING TASKS
    // This waits for both tasks to finish
    // In practice, they'll run until interrupted
    tokio::try_join!(ws_handle, processor_handle)
        .context("Error in task execution")?;
    
    info!("Pipeline stopped");
    Ok(())
}