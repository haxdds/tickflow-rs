# Tickflow

Tickflow is an async-first data pipeline for streaming market data into downstream systems. It ships with Alpaca Market Data websocket and Yahoo Finance sources, plus a PostgreSQL sink wired together with a bounded single-producer/single-consumer (SPSC) channel so you can ingest ticks with backpressure-aware processing.

## Features

- Integrates with Alpaca's streaming API and Yahoo Finance using resilient async clients.
- Persists bars, quotes, and trades to PostgreSQL with schema bootstrapping baked in.
- Fluent builder (`TickflowBuilder`) for composing sources and sinks with configurable channel sizing.
- Reusable messaging traits to plug in custom producers, processors, or destinations.

## Getting Started

1. Install Rust (edition 2024 or newer) via [rustup](https://rustup.rs/).
2. Ensure you have PostgreSQL running and reachable.
3. For Alpaca: collect API credentials (key, secret, websocket URL). For Yahoo: no credentials needed.
4. Clone the repository and enable the default feature set (`alpaca`, `postgres`, `yahoo`):

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

### Run the example pipelines

**Alpaca example:** Requires API keys (`APCA_API_KEY_ID` and `APCA_API_SECRET_KEY`) in your environment or `.env` file:

```bash
cargo run --release --example alpaca_to_postgres --features "alpaca postgres"
```

**Yahoo Finance example:** No credentials needed:

```bash
cargo run --release --example yahoo_to_postgres --features "yahoo postgres"
```

### Embed Tickflow in your own project

Re-use the builder and message traits to compose custom pipelines or reuse the provided sink/source pair:

```rust
use tickflow::config::AppConfig;
use tickflow::connectors::alpaca::websocket::AlpacaWebSocketClient;
use tickflow::prelude::*;
use tickflow::storage::{Database, postgres_handler::alpaca::AlpacaMessageHandler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let database = Database::connect(&config.database_url, AlpacaMessageHandler).await?;
    database.initialize_schema().await?;

    let websocket = AlpacaWebSocketClient::new(
        &config.alpaca_ws_url,
        &config.alpaca_api_key,
        &config.alpaca_api_secret,
        &[],
        &["ETH/USD"],
        &[],
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

