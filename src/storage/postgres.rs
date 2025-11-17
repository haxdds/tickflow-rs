//! PostgreSQL-backed message sink for market data.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};

use crate::core::{Message, MessageBatch, MessageSink};

// Type aliases to reduce verbosity
type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;
type AsyncDbResult<T> = Pin<Box<dyn Future<Output = Result<T, tokio_postgres::Error>> + Send>>;

/// Handles database operations for a specific message type.
/// Each message type implements this to define its schema and insertion logic.
pub trait DatabaseMessageHandler<M: Message>: Send + Sync + 'static {
    /// Initialize the database schema for this message type.
    fn initialize_schema(&self, client: Arc<Client>) -> AsyncDbResult<()>;

    /// Insert a batch of messages into the database.
    fn insert_batch(&self, client: Arc<Client>, batch: Vec<M>) -> AsyncResult<()>;
}

/// PostgreSQL database sink for market data messages.
pub struct Database<M: Message> {
    client: Arc<Client>,
    handler: Box<dyn DatabaseMessageHandler<M>>,
}

impl<M: Message> Database<M> {
    /// Connect to PostgreSQL and return a new Database instance.
    pub async fn connect(
        connection_string: &str,
        handler: Box<dyn DatabaseMessageHandler<M>>,
    ) -> Result<Self, tokio_postgres::Error> {
        info!("Connecting to database...");

        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        // Spawn connection task to handle errors
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Database connection error: {}", e);
            }
        });

        info!("Database connected successfully");

        Ok(Self {
            client: Arc::new(client),
            handler,
        })
    }

    /// Initialize the database schema for the message type.
    pub async fn initialize_schema(&self) -> Result<(), tokio_postgres::Error> {
        info!("Initializing database schema...");
        self.handler
            .initialize_schema(Arc::clone(&self.client))
            .await?;
        info!("Database schema initialized");
        Ok(())
    }
}

impl<M: Message> MessageSink<M> for Database<M> {
    fn name(&self) -> &'static str {
        "postgres"
    }

    fn handle_batch<'a>(
        &'a self,
        batch: MessageBatch<M>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        let client = Arc::clone(&self.client);
        Box::pin(async move {
            self.handler.insert_batch(client, batch).await
        })
    }
}

// Re-export message handlers
#[cfg(feature = "postgres")]
pub use crate::storage::postgres_handler::alpaca::AlpacaMessageHandler;
#[cfg(feature = "postgres")]
pub use crate::storage::postgres_handler::yahoo::YahooMessageHandler;
