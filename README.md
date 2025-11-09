# Tickflow

Tickflow is an async-first data pipeline for streaming market data into downstream systems. It ships with an Alpaca Market Data websocket source and a PostgreSQL sink wired together with a bounded single-producer/single-consumer (SPSC) channel so you can ingest ticks with backpressure-aware processing.

## Features

- Integrates directly with Alpaca's streaming API using a resilient Tokio/WebSocket client.
- Persists bars, quotes, and trades to PostgreSQL with schema bootstrapping baked in.
- Fluent builder (`TickflowBuilder`) for composing sources and sinks with configurable channel sizing.
- Reusable messaging traits to plug in custom producers, processors, or destinations.

## Getting Started

1. Install Rust (edition 2024 or newer) via [rustup](https://rustup.rs/).
2. Ensure you have PostgreSQL running and reachable.
3. Collect Alpaca Market Data credentials (key, secret, websocket URL).
4. Clone the repository and enable the default feature set (`alpaca`, `postgres`):

```bash
git clone https://github.com/your-org/tickflow-rs.git
cd tickflow-rs
cargo build
```

## Configuration

Tickflow reads runtime configuration from environment variables (use a `.env` file with `dotenvy` if desired):

```bash
# Required
DATABASE_URL=postgres://user:password@localhost:5432/tickflow
APCA_API_KEY_ID=your-key
APCA_API_SECRET_KEY=your-secret
APCA_WS_URL=wss://stream.data.alpaca.markets/v1beta3/crypto/us-1

# Optional
DATAFEED_CHANNEL_SIZE=2000
```

`DATAFEED_CHANNEL_SIZE` defaults to `1000` when omitted.

## Usage Examples

### Run the bundled CLI

The `tickflow` binary wires the Alpaca websocket to PostgreSQL and initializes tables on startup:

```bash
cargo run --release --bin tickflow
```

### Run the example pipeline

**Note:** You will need to provide your own Alpaca API keys (`APCA_API_KEY_ID` and `APCA_API_SECRET_KEY`) in your environment or `.env` file to access Alpaca's data streams.


The example demonstrates the full flow and is guarded by the `alpaca` and `postgres` feature flags:

```bash
cargo run --release --example alpaca_to_postgres --features "alpaca postgres"
```

### Embed Tickflow in your own project

Re-use the builder and message traits to compose custom pipelines or reuse the provided sink/source pair:

```rust
use tickflow::config::AppConfig;
use tickflow::connectors::alpaca::AlpacaWebSocketClient;
use tickflow::prelude::*;
use tickflow::storage::Database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let database = Database::connect(&config.database_url).await?;
    database.initialize_schema().await?;

    let websocket = AlpacaWebSocketClient::new(
        &config.alpaca_ws_url,
        &config.alpaca_api_key,
        &config.alpaca_api_secret,
    );

    let handles = TickflowBuilder::new(websocket, database)
        .channel_capacity(config.channel_capacity)
        .start()
        .await?;

    tokio::try_join!(handles.source, handles.processor)?;
    Ok(())
}
```

## Development

- Format and lint: `cargo fmt && cargo clippy`
- Run tests: `cargo test`
- Explore alternate sinks or sources by implementing the `MessageSource` and `MessageSink` traits.

## License

Distributed under the MIT License. See `LICENSE` for details.

