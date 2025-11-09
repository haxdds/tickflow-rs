//! Alpaca market data websocket source implementation.

use std::future::Future;
use std::pin::Pin;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};
use tracing::{debug, error, info};

use crate::core::{MessageBatch, MessageSource};

use super::types::AlpacaMessage;

type AlpacaSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type AlpacaStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Streams Alpaca market data over a websocket and yields message batches.
pub struct AlpacaWebSocketClient {
    url: String,
    api_key: String,
    api_secret: String,
    bars: Vec<String>,
    quotes: Vec<String>,
    trades: Vec<String>,
    write: Option<AlpacaSink>,
    read: Option<AlpacaStream>,
}

impl MessageSource<AlpacaMessage> for AlpacaWebSocketClient {
    fn run<'a>(
        &'a mut self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<AlpacaMessage>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.connect().await?;
            self.authenticate().await?;
            self.subscribe(
                self.bars.clone(),
                self.quotes.clone(),
                self.trades.clone(),
            )
            .await?;
            self.stream_messages(tx).await?;
            Ok(())
        })
    }
}

impl AlpacaWebSocketClient {
    /// Creates a new websocket client configured with Alpaca credentials.
    pub fn new(
        url: &str,
        api_key: &str,
        api_secret: &str,
        bars: &[&str],
        quotes: &[&str],
        trades: &[&str],
    ) -> Self {
        Self {
            url: url.to_string(),
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            bars: bars.iter().map(|s| s.to_string()).collect(),
            quotes: quotes.iter().map(|s| s.to_string()).collect(),
            trades: trades.iter().map(|s| s.to_string()).collect(),
            write: None,
            read: None,
        }
    }

    /// Establishes the websocket connection and stores split read/write halves.
    pub async fn connect(&mut self) -> Result<(), WsError> {
        info!("Try connect to websocket");
        let (ws_stream, _) = connect_async(&self.url).await?;
        info!("WebSocket connected");
        let (write, read) = ws_stream.split();

        self.write = Some(write);
        self.read = Some(read);
        Ok(())
    }

    /// Closes the websocket connection if it is currently open.
    pub async fn disconnect(&mut self) -> Result<(), WsError> {
        if self.write.is_some() {
            let mut write = self.write.take().unwrap();
            let _ = write.send(Message::Close(None)).await;
            let _ = write.close().await;
        }
        self.read = None;
        Ok(())
    }

    /// Sends the authentication payload required by Alpaca.
    pub async fn authenticate(&mut self) -> Result<(), WsError> {
        let payload = json!({
            "action": "auth",
            "key": self.api_key,
            "secret": self.api_secret
        });
        info!("Authenticating");
        self.send(Message::Text(payload.to_string())).await
    }

    async fn send(&mut self, message: Message) -> Result<(), WsError> {
        match self.write.as_mut() {
            Some(sink) => sink.send(message).await,
            None => Err(WsError::ConnectionClosed),
        }
    }

    /// Subscribes to provided channel lists, sending a single request to Alpaca.
    pub async fn subscribe(
        &mut self,
        bars: Vec<String>,
        quotes: Vec<String>,
        trades: Vec<String>,
    ) -> Result<(), WsError> {
        let payload = json!({
            "action": "subscribe",
            "bars": bars,
            "quotes": quotes,
            "trades": trades
        });

        self.send(Message::Text(payload.to_string())).await
    }

    /// Streams incoming websocket messages and forwards parsed batches to the pipeline.
    pub async fn stream_messages(
        &mut self,
        tx: tokio::sync::mpsc::Sender<MessageBatch<AlpacaMessage>>,
    ) -> Result<(), WsError> {
        info!("Taking read stream...");
        let mut read = match self.read.take() {
            Some(read) => read,
            None => panic!("There was an error streaming"),
        };

        info!("Watching read stream...");
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    info!("message: {},", &text);
                    if let Ok(parsed) = serde_json::from_str::<Vec<AlpacaMessage>>(&text) {
                        let _ = tx.send(parsed).await;
                    } else {
                        debug!("Failed to parse message");
                    }
                }
                Ok(Message::Binary(_)) => debug!("Binary message ignored"),
                Ok(Message::Ping(data)) => {
                    debug!("Received ping, sending pong");
                    if self.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Pong(_)) => debug!("Received pong"),
                Ok(Message::Close(frame)) => {
                    info!("Received close message: {:?}", frame);
                    break;
                }
                Ok(Message::Frame(_)) => {}
                Err(err) => {
                    error!("WebSocket error: {err}");
                    break;
                }
            }
        }

        self.read = Some(read);

        Ok(())
    }
}
