<!-- Current module audit generated on 2025-11-09 -->
# Current Module Audit

## `src/config.rs`
- Loads runtime settings (`DATABASE_URL`, Alpaca credentials, channel capacity) from environment variables.
- Provides `AppConfig::from_env()` returning configuration data; consumers read fields directly.

## `src/core/`
- `core::traits` hosts the foundational abstraction layer (`Message`, `MessageBatch`, `MessageSource`, `MessageSink`).
- `core::mod` re-exports the traits for consumers.

## `src/pipeline/`
- `pipeline::datafeed` contains the concrete single-producer/single-consumer orchestration (`SPSCDataFeed`, `SPSCDataFeedHandles`).
- `pipeline::processor` wraps sinks with `MessageProcessor` for channel-driven execution.
- `pipeline::builder` provides `TickflowBuilder` for ergonomic assembly of sources, sinks, and channel configuration.

## `src/prelude.rs`
- Convenience re-exports of core traits and the main pipeline entry points for easy downstream consumption.

## `src/connectors/alpaca/`
- `types` defines Alpaca WebSocket payload models (`AlpacaMessage`, `Bar`, `Quote`, `Trade`) and implements `Message`.
- `websocket` implements `AlpacaWebSocketClient`, an Alpaca-specific `MessageSource`.
- `mod.rs` exposes the Alpaca connector surface (enabled via the `alpaca` feature flag).

## `src/storage/postgres.rs`
- PostgreSQL-backed `Database` implementing `MessageSink<AlpacaMessage>` with batched persistence helpers and schema initialization.

## `src/bin/tickflow.rs`
- Feature-gated binary using the library API: loads `AppConfig`, wires Alpaca → Postgres via `TickflowBuilder`, and drives the async runtime.

