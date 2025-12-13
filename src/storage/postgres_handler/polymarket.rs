//! PostgreSQL handler for PolymarketMessage.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use chrono::NaiveDateTime;
use tokio_postgres::Client;
use tracing::{error, info};

use crate::connectors::polymarket::types::{Market, PolymarketMessage};
use crate::storage::postgres::DatabaseMessageHandler;

pub struct PolymarketMessageHandler;

impl DatabaseMessageHandler<PolymarketMessage> for PolymarketMessageHandler {
    fn initialize_schema(
        &self,
        client: Arc<Client>,
    ) -> Pin<Box<dyn Future<Output = Result<(), tokio_postgres::Error>> + Send>> {
        Box::pin(async move {
            client
                .execute(
                    "CREATE TABLE IF NOT EXISTS polymarket_markets (
                        id SERIAL PRIMARY KEY,
                        condition_id VARCHAR(66) NOT NULL UNIQUE,
                        question_id VARCHAR(66),
                        market_slug VARCHAR(255),
                        question TEXT,
                        description TEXT,
                        -- Boolean flags
                        active BOOLEAN,
                        closed BOOLEAN,
                        archived BOOLEAN,
                        accepting_orders BOOLEAN,
                        enable_order_book BOOLEAN,
                        neg_risk BOOLEAN,
                        -- Timestamps
                        end_date_iso TIMESTAMP,
                        game_start_time TIMESTAMP,
                        accepting_order_timestamp TIMESTAMP,
                        -- Numeric fields
                        minimum_order_size DOUBLE PRECISION,
                        minimum_tick_size DOUBLE PRECISION,
                        maker_base_fee DOUBLE PRECISION,
                        taker_base_fee DOUBLE PRECISION,
                        seconds_delay INTEGER,
                        -- JSONB for nested structures
                        tokens JSONB,
                        rewards JSONB,
                        tags JSONB,
                        -- Metadata
                        icon TEXT,
                        image TEXT,
                        fpmm VARCHAR(66),
                        neg_risk_market_id VARCHAR(66),
                        neg_risk_request_id VARCHAR(66),
                        notifications_enabled BOOLEAN,
                        is_50_50_outcome BOOLEAN,
                        received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    &[],
                )
                .await?;

            Ok(())
        })
    }

    fn insert_batch(
        &self,
        client: Arc<Client>,
        batch: Vec<PolymarketMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            for message in batch {
                match message {
                    PolymarketMessage::Market(market) => {
                        if let Err(e) = insert_market(&client, market).await {
                            error!(error = %e, "Failed to insert market");
                        }
                    }
                }
            }
            Ok(())
        })
    }
}

/// Parse an optional ISO timestamp string to NaiveDateTime
fn parse_timestamp(value: Option<&str>) -> Option<NaiveDateTime> {
    value.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.naive_utc())
    })
}

async fn insert_market(client: &Client, market: Market) -> Result<(), tokio_postgres::Error> {
    let end_date_iso = parse_timestamp(market.end_date_iso.as_deref());
    let game_start_time = parse_timestamp(market.game_start_time.as_deref());
    let accepting_order_timestamp = parse_timestamp(market.accepting_order_timestamp.as_deref());

    info!(
        condition_id = %market.condition_id,
        market_slug = ?market.market_slug,
        "Inserting polymarket market"
    );

    client
        .execute(
            "INSERT INTO polymarket_markets (
                condition_id, question_id, market_slug, question, description,
                active, closed, archived, accepting_orders, enable_order_book, neg_risk,
                end_date_iso, game_start_time, accepting_order_timestamp,
                minimum_order_size, minimum_tick_size, maker_base_fee, taker_base_fee, seconds_delay,
                tokens, rewards, tags,
                icon, image, fpmm, neg_risk_market_id, neg_risk_request_id,
                notifications_enabled, is_50_50_outcome
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10, $11,
                $12, $13, $14,
                $15, $16, $17, $18, $19,
                $20, $21, $22,
                $23, $24, $25, $26, $27,
                $28, $29
            )
            ON CONFLICT (condition_id) DO UPDATE SET
                question_id = EXCLUDED.question_id,
                market_slug = EXCLUDED.market_slug,
                question = EXCLUDED.question,
                description = EXCLUDED.description,
                active = EXCLUDED.active,
                closed = EXCLUDED.closed,
                archived = EXCLUDED.archived,
                accepting_orders = EXCLUDED.accepting_orders,
                enable_order_book = EXCLUDED.enable_order_book,
                neg_risk = EXCLUDED.neg_risk,
                end_date_iso = EXCLUDED.end_date_iso,
                game_start_time = EXCLUDED.game_start_time,
                accepting_order_timestamp = EXCLUDED.accepting_order_timestamp,
                minimum_order_size = EXCLUDED.minimum_order_size,
                minimum_tick_size = EXCLUDED.minimum_tick_size,
                maker_base_fee = EXCLUDED.maker_base_fee,
                taker_base_fee = EXCLUDED.taker_base_fee,
                seconds_delay = EXCLUDED.seconds_delay,
                tokens = EXCLUDED.tokens,
                rewards = EXCLUDED.rewards,
                tags = EXCLUDED.tags,
                icon = EXCLUDED.icon,
                image = EXCLUDED.image,
                fpmm = EXCLUDED.fpmm,
                neg_risk_market_id = EXCLUDED.neg_risk_market_id,
                neg_risk_request_id = EXCLUDED.neg_risk_request_id,
                notifications_enabled = EXCLUDED.notifications_enabled,
                is_50_50_outcome = EXCLUDED.is_50_50_outcome,
                received_at = CURRENT_TIMESTAMP",
            &[
                &market.condition_id,
                &market.question_id,
                &market.market_slug,
                &market.question,
                &market.description,
                &market.active,
                &market.closed,
                &market.archived,
                &market.accepting_orders,
                &market.enable_order_book,
                &market.neg_risk,
                &end_date_iso,
                &game_start_time,
                &accepting_order_timestamp,
                &market.minimum_order_size,
                &market.minimum_tick_size,
                &market.maker_base_fee,
                &market.taker_base_fee,
                &market.seconds_delay,
                &market.tokens,
                &market.rewards,
                &market.tags,
                &market.icon,
                &market.image,
                &market.fpmm,
                &market.neg_risk_market_id,
                &market.neg_risk_request_id,
                &market.notifications_enabled,
                &market.is_50_50_outcome,
            ],
        )
        .await?;

    Ok(())
}
