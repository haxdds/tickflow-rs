// Learning Goals:
// - tokio::sync::mpsc for async channels
// - Receiving from channels in a loop
// - Pattern matching on enum variants
// - Graceful shutdown handling

use crate::database::Database;
use crate::messages::AlpacaMessage;
use anyhow::Result;
use chrono::DateTime;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Message processor that receives market data and stores it
///
/// Rust Concept: WORKER PATTERN
/// This struct acts as a worker that processes messages from a channel
// /// It decouples message receipt (WebSocket) from processing (database writes)
// pub struct MessageProcessor {
//     database: Database,
// }
use crate::message_types::{Message, MessageBatch, MessageSink};

pub struct MessageProcessor<M: Message> {
    sink: std::sync::Arc<dyn MessageSink<M>>,
}

impl<M: Message> MessageProcessor<M> {
    pub fn new<S>(sink: S) -> Self
    where
        S: MessageSink<M>,
    {
        Self { sink: std::sync::Arc::new(sink) }
    }

    pub async fn process_messages(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<MessageBatch<M>>,
    ) -> anyhow::Result<()> {
        tracing::info!("Message processor started ({})", self.sink.name());
        while let Some(batch) = rx.recv().await {
            if let Err(err) = self.sink.handle_batch(batch).await {
                tracing::warn!("{} sink error: {err}", self.sink.name());
            }
        }
        tracing::info!("Message processor stopped");
        Ok(())
    }
}

// impl MessageProcessor {
//     /// Create a new message processor
//     pub fn new(database: Database) -> Self {
//         Self { database }
//     }

//     /// Start processing messages from the channel
//     ///
//     /// Learning: CHANNEL RECEIVER
//     /// The mpsc::Receiver<T> is the receiving end of a channel
//     /// We can receive messages with .recv().await
//     ///
//     /// Learning: OWNERSHIP
//     /// We take ownership of 'rx' (not &rx)
//     /// This ensures only this function can receive from this channel
//     pub async fn process_messages(&self, mut rx: mpsc::Receiver<Vec<AlpacaMessage>>) -> Result<()> {
//         info!("Message processor started");

//         // Process messages until channel is closed
//         // Learning: WHILE LET LOOP
//         // This is idiomatic Rust for consuming from a channel
//         // The loop ends when rx.recv() returns None (channel closed)
//         while let Some(message) = rx.recv().await {
//             info!("Processor got message!");
//             for m in message {
//                 // debug!(m);
//                 if let Err(e) = self.handle_message(m).await {
//                     warn!("Error handling message: {}", e);
//                     // Continue processing despite errors
//                 }
//             }
//         }

//         info!("Message processor stopped (channel closed)");
//         Ok(())
//     }

//     /// Handle a single message
//     ///
//     /// Learning: PATTERN MATCHING ON ENUMS
//     /// Rust's match expression is exhaustive - we must handle all cases
//     /// Each variant can extract its inner data
//     async fn handle_message(&self, message: AlpacaMessage) -> Result<()> {
//         match message {
//             // Connection messages
//             AlpacaMessage::Success { msg } => {
//                 info!("✓ Success: {}", msg);
//             }

//             AlpacaMessage::Error { code, msg } => {
//                 warn!("✗ Error {}: {}", code, msg);
//             }

//             AlpacaMessage::Subscription {
//                 trades,
//                 quotes,
//                 bars,
//                 ..
//             } => {
//                 info!("📊 Subscriptions updated:");
//                 if !bars.is_empty() {
//                     info!("  Bars: {:?}", bars);
//                 }
//                 if !quotes.is_empty() {
//                     info!("  Quotes: {:?}", quotes);
//                 }
//                 if !trades.is_empty() {
//                     info!("  Trades: {:?}", trades);
//                 }
//             }

//             // Market data messages
//             AlpacaMessage::Bar(bar) => {
//                 debug!(
//                     "📊 BAR: {} O:{} H:{} L:{} C:{} V:{} ({:.2}%)",
//                     bar.symbol,
//                     bar.open,
//                     bar.high,
//                     bar.low,
//                     bar.close,
//                     bar.volume,
//                     bar.price_change_percent()
//                 );
//                 let ts = DateTime::parse_from_rfc3339(&bar.timestamp)?.naive_utc();
//                 let trade_count = bar.trade_count.map(|count| count as i64);
//                 // Store in database
//                 self.database
//                     .insert_bar(
//                         &bar.symbol,
//                         bar.open,
//                         bar.high,
//                         bar.low,
//                         bar.close,
//                         bar.volume as i64,
//                         ts,
//                         &trade_count,
//                         &bar.vwap,
//                     )
//                     .await?;
//             }

//             AlpacaMessage::Quote(quote) => {
//                 debug!(
//                     "💱 QUOTE: {} Bid:{} Ask:{} Spread:{:.4} ({:.2} bps)",
//                     quote.symbol,
//                     quote.bid_price,
//                     quote.ask_price,
//                     quote.spread(),
//                     quote.spread_bps()
//                 );
//                 let ts = DateTime::parse_from_rfc3339(&quote.timestamp)?.naive_utc();
//                 // Store in database
//                 self.database
//                     .insert_quote(
//                         &quote.symbol,
//                         &quote.bid_exchange,
//                         quote.bid_price,
//                         quote.bid_size as i64,
//                         &quote.ask_exchange,
//                         quote.ask_price,
//                         quote.ask_size as i64,
//                         ts,
//                         &quote.tape,
//                     )
//                     .await?;
//             }

//             AlpacaMessage::Trade(trade) => {
//                 debug!(
//                     "💸 TRADE: {} Price:{} Size:{}",
//                     trade.symbol, trade.price, trade.size
//                 );
//                 let ts = DateTime::parse_from_rfc3339(&trade.timestamp)?.naive_utc();
//                 // Store in database
//                 self.database
//                     .insert_trade(
//                         trade.id as i64,
//                         &trade.symbol,
//                         &trade.exchange,
//                         trade.price,
//                         trade.size as i64,
//                         ts,
//                         &trade.tape,
//                         &trade.tks,
//                     )
//                     .await?;
//             }
//         }

//         Ok(())
//     }
// }
