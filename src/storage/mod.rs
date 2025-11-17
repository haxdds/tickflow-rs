//! Storage integrations for message sinks.

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "postgres")]
pub mod postgres_handler;

#[cfg(feature = "postgres")]
pub use postgres::Database;
