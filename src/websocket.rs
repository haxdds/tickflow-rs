// #![allow(unused_imports)]

use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use serde_json::json;
use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream};
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::MaybeTlsStream;
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

use crate::messages::AlpacaMessage;

type AlpacaSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type AlpacaStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct AlpacaWebSocketClient {
    url: String,
    api_key: String,
    api_secret: String,
    write: Option<AlpacaSink>,
    read: Option<AlpacaStream>,
}

impl AlpacaWebSocketClient {
    pub fn new(url: &str, api_key: &str, api_secret: &str) -> Self {
        Self {
            url: url.to_string(),
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            write: None,
            read: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), WsError> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url).await?;
        info!("WebSocket connected");
        let (write, read) = ws_stream.split();
        
        self.write = Some(write);
        self.read = Some(read);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), WsError> {
        // Always try to close the websocket connection, even if it's already closed
        if self.write.is_some() {
            // Take the write part out so we can use it
            let mut write = self.write.take().unwrap();
            // Try sending a Close message (ignore errors)
            let _ = write.send(tungstenite::protocol::Message::Close(None)).await;
            let _ = write.close().await;
        }
        self.read = None;
        Ok(())
    }

    pub async fn authenticate(&mut self) -> Result<(), tungstenite::Error> {
        let payload = json!({
            "action": "auth",
            "key": self.api_key,
            "secret": self.api_secret
        });
        
        self.send(Message::Text(payload.to_string())).await
    }

    async fn send(&mut self, message: Message) -> Result<(), WsError> {
        match self.write.as_mut() {
            Some(sink) => sink.send(message).await,
            None => Err(WsError::ConnectionClosed),
        }
    }

    pub async fn subscribe(&mut self, bars: &[&str], quotes: &[&str], trades: &[&str]) -> Result<(), WsError> {
        let payload = json!({
            "action": "subscribe",
            "bars": bars,
            "quotes": quotes,
            "trades": trades
        });
        
        self.send(Message::Text(payload.to_string())).await
    }

    pub async fn stream_messages(&mut self, tx: tokio::sync::mpsc::Sender<AlpacaMessage>) -> Result<(), WsError> {
        let mut read = match self.read.take() {
            Some(read) => read,
            None => panic!("There was an error streaming"),
        };

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(parsed) = serde_json::from_str::<AlpacaMessage>(&text) {
                        if tx.send(parsed).await.is_err() {
                            warn! { "Receiver dropped, stopping stream" };
                            break;
                        }
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
