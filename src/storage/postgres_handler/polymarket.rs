//! PostgreSQL handler for PolymarketMessage.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use tokio_postgres::Client;
use tracing::{error, info};

use crate::connectors::polymarket::types::{Market, MarketGamma, PolymarketMessage};
use crate::storage::postgres::DatabaseMessageHandler;

pub struct PolymarketMessageHandler;

impl DatabaseMessageHandler<PolymarketMessage> for PolymarketMessageHandler {
    fn initialize_schema(
        &self,
        client: Arc<Client>,
    ) -> Pin<Box<dyn Future<Output = Result<(), tokio_postgres::Error>> + Send>> {
        Box::pin(async move {
            // Create polymarket_markets table (CLOB API)
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

            // Create market_gamma table (Gamma API)
            client
                .execute(
                    "CREATE TABLE IF NOT EXISTS market_gamma (
                        id VARCHAR(255) PRIMARY KEY,
                        question TEXT NOT NULL,
                        condition_id VARCHAR(66) NOT NULL,
                        slug VARCHAR(255) NOT NULL,
                        resolution_source TEXT,
                        end_date TIMESTAMP NOT NULL,
                        liquidity TEXT,
                        start_date TIMESTAMP NOT NULL,
                        image TEXT,
                        icon TEXT,
                        description TEXT,
                        outcomes TEXT,
                        outcome_prices TEXT,
                        volume TEXT,
                        active BOOLEAN DEFAULT false,
                        closed BOOLEAN DEFAULT false,
                        market_maker_address TEXT,
                        created_at TIMESTAMP NOT NULL,
                        updated_at TIMESTAMP NOT NULL,
                        new BOOLEAN DEFAULT false,
                        featured BOOLEAN DEFAULT false,
                        submitted_by TEXT,
                        archived BOOLEAN DEFAULT false,
                        resolved_by TEXT,
                        restricted BOOLEAN DEFAULT false,
                        group_item_title TEXT,
                        group_item_threshold TEXT,
                        question_id VARCHAR(66),
                        enable_order_book BOOLEAN DEFAULT false,
                        order_price_min_tick_size DOUBLE PRECISION,
                        order_min_size DOUBLE PRECISION,
                        volume_num DOUBLE PRECISION,
                        liquidity_num DOUBLE PRECISION,
                        end_date_iso TEXT,
                        start_date_iso TEXT,
                        has_reviewed_dates BOOLEAN DEFAULT false,
                        volume_24hr DOUBLE PRECISION,
                        volume_1wk DOUBLE PRECISION,
                        volume_1mo DOUBLE PRECISION,
                        volume_1yr DOUBLE PRECISION,
                        clob_token_ids TEXT,
                        uma_bond TEXT,
                        uma_reward TEXT,
                        volume_24hr_clob DOUBLE PRECISION,
                        volume_1wk_clob DOUBLE PRECISION,
                        volume_1mo_clob DOUBLE PRECISION,
                        volume_1yr_clob DOUBLE PRECISION,
                        volume_clob DOUBLE PRECISION,
                        liquidity_clob DOUBLE PRECISION,
                        accepting_orders BOOLEAN DEFAULT false,
                        neg_risk BOOLEAN DEFAULT false,
                        events JSONB,
                        ready BOOLEAN DEFAULT false,
                        funded BOOLEAN DEFAULT false,
                        accepting_orders_timestamp TIMESTAMP,
                        cyom BOOLEAN DEFAULT false,
                        competitive DOUBLE PRECISION,
                        pager_duty_notification_enabled BOOLEAN DEFAULT false,
                        approved BOOLEAN DEFAULT false,
                        rewards_min_size DOUBLE PRECISION,
                        rewards_max_spread DOUBLE PRECISION,
                        spread DOUBLE PRECISION,
                        one_day_price_change DOUBLE PRECISION,
                        one_week_price_change DOUBLE PRECISION,
                        one_month_price_change DOUBLE PRECISION,
                        last_trade_price DOUBLE PRECISION,
                        best_bid DOUBLE PRECISION,
                        best_ask DOUBLE PRECISION,
                        automatically_active BOOLEAN DEFAULT false,
                        clear_book_on_start BOOLEAN DEFAULT false,
                        manual_activation BOOLEAN DEFAULT false,
                        neg_risk_other BOOLEAN DEFAULT false,
                        uma_resolution_statuses TEXT,
                        pending_deployment BOOLEAN DEFAULT false,
                        deploying BOOLEAN DEFAULT false,
                        rfq_enabled BOOLEAN DEFAULT false,
                        holding_rewards_enabled BOOLEAN DEFAULT false,
                        fees_enabled BOOLEAN DEFAULT false,
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
            let mut markets: Vec<Market> = Vec::new();
            let mut markets_gamma: Vec<MarketGamma> = Vec::new();

            for msg in batch {
                match msg {
                    PolymarketMessage::Market(market) => markets.push(market),
                    PolymarketMessage::MarketGamma(market_gamma) => markets_gamma.push(market_gamma),
                }
            }
    
            if !markets.is_empty() {
                if let Err(e) = insert_markets_batch(&client, markets).await {
                    error!(
                        error = %e,
                        error_debug = ?e,
                        "Failed to insert market batch"
                    );
                }
            }

            if !markets_gamma.is_empty() {
                if let Err(e) = insert_markets_gamma_batch(&client, markets_gamma).await {
                    error!(
                        error = %e,
                        error_debug = ?e,
                        "Failed to insert market_gamma batch"
                    );
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


struct MarketInsertParams {
    condition_id: String,
    question_id: Option<String>,
    market_slug: Option<String>,
    question: Option<String>,
    description: Option<String>,
    active: Option<bool>,
    closed: Option<bool>,
    archived: Option<bool>,
    accepting_orders: Option<bool>,
    enable_order_book: Option<bool>,
    neg_risk: Option<bool>,
    end_date_iso: Option<NaiveDateTime>,
    game_start_time: Option<NaiveDateTime>,
    accepting_order_timestamp: Option<NaiveDateTime>,
    minimum_order_size: Option<f64>,
    minimum_tick_size: Option<f64>,
    maker_base_fee: Option<f64>,
    taker_base_fee: Option<f64>,
    seconds_delay: Option<i32>,
    tokens: Option<serde_json::Value>,
    rewards: Option<serde_json::Value>,
    tags: Option<serde_json::Value>,
    icon: Option<String>,
    image: Option<String>,
    fpmm: Option<String>,
    neg_risk_market_id: Option<String>,
    neg_risk_request_id: Option<String>,
    notifications_enabled: Option<bool>,
    is_50_50_outcome: Option<bool>,
}

impl From<Market> for MarketInsertParams {
    fn from(market: Market) -> Self {
        Self {
            condition_id: market.condition_id,
            question_id: market.question_id,
            market_slug: market.market_slug,
            question: market.question,
            description: market.description,
            active: Some(market.active),
            closed: Some(market.closed),
            archived: Some(market.archived),
            accepting_orders: Some(market.accepting_orders),
            enable_order_book: Some(market.enable_order_book),
            neg_risk: Some(market.neg_risk),
            end_date_iso: parse_timestamp(market.end_date_iso.as_deref()),
            game_start_time: parse_timestamp(market.game_start_time.as_deref()),
            accepting_order_timestamp: parse_timestamp(market.accepting_order_timestamp.as_deref()),
            minimum_order_size: Some(market.minimum_order_size),
            minimum_tick_size: Some(market.minimum_tick_size),
            maker_base_fee: Some(market.maker_base_fee),
            taker_base_fee: Some(market.taker_base_fee),
            seconds_delay: Some(market.seconds_delay),
            tokens: Some(market.tokens),
            rewards: Some(market.rewards),
            tags: Some(market.tags),
            icon: market.icon,
            image: market.image,
            fpmm: market.fpmm,
            neg_risk_market_id: market.neg_risk_market_id,
            neg_risk_request_id: market.neg_risk_request_id,
            notifications_enabled: Some(market.notifications_enabled),
            is_50_50_outcome: Some(market.is_50_50_outcome),
        }
    }
}

async fn insert_markets_batch(
    client: &Client,
    markets: Vec<Market>,
) -> Result<(), tokio_postgres::Error> {
    if markets.is_empty() {
        return Ok(());
    }

    // Deduplicate by condition_id, keeping the last occurrence
    let mut seen = std::collections::HashMap::new();
    let mut deduped_markets = Vec::new();
    
    for market in markets {
        seen.insert(market.condition_id.clone(), market);
    }
    
    deduped_markets.extend(seen.into_values());
    
    if deduped_markets.is_empty() {
        return Ok(());
    }

    // Convert to owned params
    let params_vec: Vec<MarketInsertParams> = deduped_markets.into_iter().map(Into::into).collect();
    
    // Build multi-row INSERT
    let mut value_strings = Vec::new();
    let params_per_row = 29;
    
    for i in 0..params_vec.len() {
        let base = i * params_per_row;
        let placeholders: Vec<String> = (1..=params_per_row)
            .map(|j| format!("${}", base + j))
            .collect();
        value_strings.push(format!("({})", placeholders.join(", ")));
    }

    // Flatten all parameters
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    for p in &params_vec {
        params.push(&p.condition_id);
        params.push(&p.question_id);
        params.push(&p.market_slug);
        params.push(&p.question);
        params.push(&p.description);
        params.push(&p.active);
        params.push(&p.closed);
        params.push(&p.archived);
        params.push(&p.accepting_orders);
        params.push(&p.enable_order_book);
        params.push(&p.neg_risk);
        params.push(&p.end_date_iso);
        params.push(&p.game_start_time);
        params.push(&p.accepting_order_timestamp);
        params.push(&p.minimum_order_size);
        params.push(&p.minimum_tick_size);
        params.push(&p.maker_base_fee);
        params.push(&p.taker_base_fee);
        params.push(&p.seconds_delay);
        params.push(&p.tokens);
        params.push(&p.rewards);
        params.push(&p.tags);
        params.push(&p.icon);
        params.push(&p.image);
        params.push(&p.fpmm);
        params.push(&p.neg_risk_market_id);
        params.push(&p.neg_risk_request_id);
        params.push(&p.notifications_enabled);
        params.push(&p.is_50_50_outcome);
    }

    let query = format!(
        "INSERT INTO polymarket_markets (
            condition_id, question_id, market_slug, question, description,
            active, closed, archived, accepting_orders, enable_order_book, neg_risk,
            end_date_iso, game_start_time, accepting_order_timestamp,
            minimum_order_size, minimum_tick_size, maker_base_fee, taker_base_fee, seconds_delay,
            tokens, rewards, tags,
            icon, image, fpmm, neg_risk_market_id, neg_risk_request_id,
            notifications_enabled, is_50_50_outcome
        ) VALUES {}
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
        value_strings.join(", ")
    );

    client.execute(&query, &params).await?;

    Ok(())
}

struct MarketGammaInsertParams {
    id: String,
    question: String,
    condition_id: String,
    slug: String,
    resolution_source: Option<String>,
    end_date: NaiveDateTime,
    liquidity: Option<String>,
    start_date: NaiveDateTime,
    image: Option<String>,
    icon: Option<String>,
    description: Option<String>,
    outcomes: Option<String>,
    outcome_prices: Option<String>,
    volume: Option<String>,
    active: bool,
    closed: bool,
    market_maker_address: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    new: bool,
    featured: bool,
    submitted_by: Option<String>,
    archived: bool,
    resolved_by: Option<String>,
    restricted: bool,
    group_item_title: Option<String>,
    group_item_threshold: Option<String>,
    question_id: Option<String>,
    enable_order_book: bool,
    order_price_min_tick_size: Option<f64>,
    order_min_size: Option<f64>,
    volume_num: Option<f64>,
    liquidity_num: Option<f64>,
    end_date_iso: Option<String>,
    start_date_iso: Option<String>,
    has_reviewed_dates: bool,
    volume_24hr: Option<f64>,
    volume_1wk: Option<f64>,
    volume_1mo: Option<f64>,
    volume_1yr: Option<f64>,
    clob_token_ids: Option<String>,
    uma_bond: Option<String>,
    uma_reward: Option<String>,
    volume_24hr_clob: Option<f64>,
    volume_1wk_clob: Option<f64>,
    volume_1mo_clob: Option<f64>,
    volume_1yr_clob: Option<f64>,
    volume_clob: Option<f64>,
    liquidity_clob: Option<f64>,
    accepting_orders: bool,
    neg_risk: bool,
    events: serde_json::Value,
    ready: bool,
    funded: bool,
    accepting_orders_timestamp: Option<NaiveDateTime>,
    cyom: bool,
    competitive: Option<f64>,
    pager_duty_notification_enabled: bool,
    approved: bool,
    rewards_min_size: Option<f64>,
    rewards_max_spread: Option<f64>,
    spread: Option<f64>,
    one_day_price_change: Option<f64>,
    one_week_price_change: Option<f64>,
    one_month_price_change: Option<f64>,
    last_trade_price: Option<f64>,
    best_bid: Option<f64>,
    best_ask: Option<f64>,
    automatically_active: bool,
    clear_book_on_start: bool,
    manual_activation: bool,
    neg_risk_other: bool,
    uma_resolution_statuses: Option<String>,
    pending_deployment: bool,
    deploying: bool,
    rfq_enabled: bool,
    holding_rewards_enabled: bool,
    fees_enabled: bool,
}

impl TryFrom<MarketGamma> for MarketGammaInsertParams {
    type Error = anyhow::Error;

    fn try_from(market: MarketGamma) -> Result<Self> {
        let end_date = chrono::DateTime::parse_from_rfc3339(&market.end_date)
            .context("Failed to parse end_date")?
            .naive_utc();
        
        let start_date = chrono::DateTime::parse_from_rfc3339(&market.start_date)
            .context("Failed to parse start_date")?
            .naive_utc();
        
        let created_at = chrono::DateTime::parse_from_rfc3339(&market.created_at)
            .context("Failed to parse created_at")?
            .naive_utc();
        
        let updated_at = chrono::DateTime::parse_from_rfc3339(&market.updated_at)
            .context("Failed to parse updated_at")?
            .naive_utc();
        
        let accepting_orders_timestamp = market.accepting_orders_timestamp.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.naive_utc())
        });

        Ok(Self {
            id: market.id,
            question: market.question,
            condition_id: market.condition_id,
            slug: market.slug,
            resolution_source: market.resolution_source,
            end_date,
            liquidity: market.liquidity,
            start_date,
            image: market.image,
            icon: market.icon,
            description: market.description,
            outcomes: market.outcomes,
            outcome_prices: market.outcome_prices,
            volume: market.volume,
            active: market.active,
            closed: market.closed,
            market_maker_address: market.market_maker_address,
            created_at,
            updated_at,
            new: market.new,
            featured: market.featured,
            submitted_by: market.submitted_by,
            archived: market.archived,
            resolved_by: market.resolved_by,
            restricted: market.restricted,
            group_item_title: market.group_item_title,
            group_item_threshold: market.group_item_threshold,
            question_id: market.question_id,
            enable_order_book: market.enable_order_book,
            order_price_min_tick_size: market.order_price_min_tick_size,
            order_min_size: market.order_min_size,
            volume_num: market.volume_num,
            liquidity_num: market.liquidity_num,
            end_date_iso: market.end_date_iso,
            start_date_iso: market.start_date_iso,
            has_reviewed_dates: market.has_reviewed_dates,
            volume_24hr: market.volume_24hr,
            volume_1wk: market.volume_1wk,
            volume_1mo: market.volume_1mo,
            volume_1yr: market.volume_1yr,
            clob_token_ids: market.clob_token_ids,
            uma_bond: market.uma_bond,
            uma_reward: market.uma_reward,
            volume_24hr_clob: market.volume_24hr_clob,
            volume_1wk_clob: market.volume_1wk_clob,
            volume_1mo_clob: market.volume_1mo_clob,
            volume_1yr_clob: market.volume_1yr_clob,
            volume_clob: market.volume_clob,
            liquidity_clob: market.liquidity_clob,
            accepting_orders: market.accepting_orders,
            neg_risk: market.neg_risk,
            events: market.events,
            ready: market.ready,
            funded: market.funded,
            accepting_orders_timestamp,
            cyom: market.cyom,
            competitive: market.competitive,
            pager_duty_notification_enabled: market.pager_duty_notification_enabled,
            approved: market.approved,
            rewards_min_size: market.rewards_min_size,
            rewards_max_spread: market.rewards_max_spread,
            spread: market.spread,
            one_day_price_change: market.one_day_price_change,
            one_week_price_change: market.one_week_price_change,
            one_month_price_change: market.one_month_price_change,
            last_trade_price: market.last_trade_price,
            best_bid: market.best_bid,
            best_ask: market.best_ask,
            automatically_active: market.automatically_active,
            clear_book_on_start: market.clear_book_on_start,
            manual_activation: market.manual_activation,
            neg_risk_other: market.neg_risk_other,
            uma_resolution_statuses: market.uma_resolution_statuses,
            pending_deployment: market.pending_deployment,
            deploying: market.deploying,
            rfq_enabled: market.rfq_enabled,
            holding_rewards_enabled: market.holding_rewards_enabled,
            fees_enabled: market.fees_enabled,
        })
    }
}

async fn insert_markets_gamma_batch(
    client: &Client,
    markets: Vec<MarketGamma>,
) -> Result<(), tokio_postgres::Error> {
    if markets.is_empty() {
        return Ok(());
    }

    // Deduplicate by id, keeping the last occurrence
    let mut seen = std::collections::HashMap::new();
    
    for market in markets {
        seen.insert(market.id.clone(), market);
    }
    
    let deduped_markets: Vec<MarketGamma> = seen.into_values().collect();
    
    if deduped_markets.is_empty() {
        return Ok(());
    }

    // Convert to owned params, filtering out any that fail to parse
    let params_vec: Vec<MarketGammaInsertParams> = deduped_markets
        .into_iter()
        .filter_map(|market| {
            match MarketGammaInsertParams::try_from(market) {
                Ok(params) => Some(params),
                Err(e) => {
                    error!(error = %e, "Failed to convert MarketGamma to insert params");
                    None
                }
            }
        })
        .collect();
    
    if params_vec.is_empty() {
        return Ok(());
    }

    // Process in chunks to avoid "value too large to transmit" error
    const CHUNK_SIZE: usize = 100;
    const PARAMS_PER_ROW: usize = 78;
    let mut total_inserted = 0;

    for chunk in params_vec.chunks(CHUNK_SIZE) {
        // Build multi-row INSERT for this chunk
        let mut value_strings = Vec::new();
        
        for i in 0..chunk.len() {
            let base = i * PARAMS_PER_ROW;
            let placeholders: Vec<String> = (1..=PARAMS_PER_ROW)
                .map(|j| format!("${}", base + j))
                .collect();
            value_strings.push(format!("({})", placeholders.join(", ")));
        }

        // Flatten all parameters for this chunk
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        for p in chunk {
            params.push(&p.id);
            params.push(&p.question);
            params.push(&p.condition_id);
            params.push(&p.slug);
            params.push(&p.resolution_source);
            params.push(&p.end_date);
            params.push(&p.liquidity);
            params.push(&p.start_date);
            params.push(&p.image);
            params.push(&p.icon);
            params.push(&p.description);
            params.push(&p.outcomes);
            params.push(&p.outcome_prices);
            params.push(&p.volume);
            params.push(&p.active);
            params.push(&p.closed);
            params.push(&p.market_maker_address);
            params.push(&p.created_at);
            params.push(&p.updated_at);
            params.push(&p.new);
            params.push(&p.featured);
            params.push(&p.submitted_by);
            params.push(&p.archived);
            params.push(&p.resolved_by);
            params.push(&p.restricted);
            params.push(&p.group_item_title);
            params.push(&p.group_item_threshold);
            params.push(&p.question_id);
            params.push(&p.enable_order_book);
            params.push(&p.order_price_min_tick_size);
            params.push(&p.order_min_size);
            params.push(&p.volume_num);
            params.push(&p.liquidity_num);
            params.push(&p.end_date_iso);
            params.push(&p.start_date_iso);
            params.push(&p.has_reviewed_dates);
            params.push(&p.volume_24hr);
            params.push(&p.volume_1wk);
            params.push(&p.volume_1mo);
            params.push(&p.volume_1yr);
            params.push(&p.clob_token_ids);
            params.push(&p.uma_bond);
            params.push(&p.uma_reward);
            params.push(&p.volume_24hr_clob);
            params.push(&p.volume_1wk_clob);
            params.push(&p.volume_1mo_clob);
            params.push(&p.volume_1yr_clob);
            params.push(&p.volume_clob);
            params.push(&p.liquidity_clob);
            params.push(&p.accepting_orders);
            params.push(&p.neg_risk);
            params.push(&p.events);
            params.push(&p.ready);
            params.push(&p.funded);
            params.push(&p.accepting_orders_timestamp);
            params.push(&p.cyom);
            params.push(&p.competitive);
            params.push(&p.pager_duty_notification_enabled);
            params.push(&p.approved);
            params.push(&p.rewards_min_size);
            params.push(&p.rewards_max_spread);
            params.push(&p.spread);
            params.push(&p.one_day_price_change);
            params.push(&p.one_week_price_change);
            params.push(&p.one_month_price_change);
            params.push(&p.last_trade_price);
            params.push(&p.best_bid);
            params.push(&p.best_ask);
            params.push(&p.automatically_active);
            params.push(&p.clear_book_on_start);
            params.push(&p.manual_activation);
            params.push(&p.neg_risk_other);
            params.push(&p.uma_resolution_statuses);
            params.push(&p.pending_deployment);
            params.push(&p.deploying);
            params.push(&p.rfq_enabled);
            params.push(&p.holding_rewards_enabled);
            params.push(&p.fees_enabled);
        }

        let query = format!(
            "INSERT INTO market_gamma (
                id, question, condition_id, slug, resolution_source, end_date, liquidity, start_date,
                image, icon, description, outcomes, outcome_prices, volume, active, closed,
                market_maker_address, created_at, updated_at, new, featured, submitted_by, archived,
                resolved_by, restricted, group_item_title, group_item_threshold, question_id,
                enable_order_book, order_price_min_tick_size, order_min_size, volume_num, liquidity_num,
                end_date_iso, start_date_iso, has_reviewed_dates, volume_24hr, volume_1wk, volume_1mo,
                volume_1yr, clob_token_ids, uma_bond, uma_reward, volume_24hr_clob, volume_1wk_clob,
                volume_1mo_clob, volume_1yr_clob, volume_clob, liquidity_clob, accepting_orders,
                neg_risk, events, ready, funded, accepting_orders_timestamp, cyom, competitive,
                pager_duty_notification_enabled, approved, rewards_min_size, rewards_max_spread, spread,
                one_day_price_change, one_week_price_change, one_month_price_change, last_trade_price,
                best_bid, best_ask, automatically_active, clear_book_on_start, manual_activation,
                neg_risk_other, uma_resolution_statuses, pending_deployment, deploying, rfq_enabled,
                holding_rewards_enabled, fees_enabled
            ) VALUES {}
            ON CONFLICT (id) DO UPDATE SET
                question = EXCLUDED.question,
                condition_id = EXCLUDED.condition_id,
                slug = EXCLUDED.slug,
                resolution_source = EXCLUDED.resolution_source,
                end_date = EXCLUDED.end_date,
                liquidity = EXCLUDED.liquidity,
                start_date = EXCLUDED.start_date,
                image = EXCLUDED.image,
                icon = EXCLUDED.icon,
                description = EXCLUDED.description,
                outcomes = EXCLUDED.outcomes,
                outcome_prices = EXCLUDED.outcome_prices,
                volume = EXCLUDED.volume,
                active = EXCLUDED.active,
                closed = EXCLUDED.closed,
                market_maker_address = EXCLUDED.market_maker_address,
                created_at = EXCLUDED.created_at,
                updated_at = EXCLUDED.updated_at,
                new = EXCLUDED.new,
                featured = EXCLUDED.featured,
                submitted_by = EXCLUDED.submitted_by,
                archived = EXCLUDED.archived,
                resolved_by = EXCLUDED.resolved_by,
                restricted = EXCLUDED.restricted,
                group_item_title = EXCLUDED.group_item_title,
                group_item_threshold = EXCLUDED.group_item_threshold,
                question_id = EXCLUDED.question_id,
                enable_order_book = EXCLUDED.enable_order_book,
                order_price_min_tick_size = EXCLUDED.order_price_min_tick_size,
                order_min_size = EXCLUDED.order_min_size,
                volume_num = EXCLUDED.volume_num,
                liquidity_num = EXCLUDED.liquidity_num,
                end_date_iso = EXCLUDED.end_date_iso,
                start_date_iso = EXCLUDED.start_date_iso,
                has_reviewed_dates = EXCLUDED.has_reviewed_dates,
                volume_24hr = EXCLUDED.volume_24hr,
                volume_1wk = EXCLUDED.volume_1wk,
                volume_1mo = EXCLUDED.volume_1mo,
                volume_1yr = EXCLUDED.volume_1yr,
                clob_token_ids = EXCLUDED.clob_token_ids,
                uma_bond = EXCLUDED.uma_bond,
                uma_reward = EXCLUDED.uma_reward,
                volume_24hr_clob = EXCLUDED.volume_24hr_clob,
                volume_1wk_clob = EXCLUDED.volume_1wk_clob,
                volume_1mo_clob = EXCLUDED.volume_1mo_clob,
                volume_1yr_clob = EXCLUDED.volume_1yr_clob,
                volume_clob = EXCLUDED.volume_clob,
                liquidity_clob = EXCLUDED.liquidity_clob,
                accepting_orders = EXCLUDED.accepting_orders,
                neg_risk = EXCLUDED.neg_risk,
                events = EXCLUDED.events,
                ready = EXCLUDED.ready,
                funded = EXCLUDED.funded,
                accepting_orders_timestamp = EXCLUDED.accepting_orders_timestamp,
                cyom = EXCLUDED.cyom,
                competitive = EXCLUDED.competitive,
                pager_duty_notification_enabled = EXCLUDED.pager_duty_notification_enabled,
                approved = EXCLUDED.approved,
                rewards_min_size = EXCLUDED.rewards_min_size,
                rewards_max_spread = EXCLUDED.rewards_max_spread,
                spread = EXCLUDED.spread,
                one_day_price_change = EXCLUDED.one_day_price_change,
                one_week_price_change = EXCLUDED.one_week_price_change,
                one_month_price_change = EXCLUDED.one_month_price_change,
                last_trade_price = EXCLUDED.last_trade_price,
                best_bid = EXCLUDED.best_bid,
                best_ask = EXCLUDED.best_ask,
                automatically_active = EXCLUDED.automatically_active,
                clear_book_on_start = EXCLUDED.clear_book_on_start,
                manual_activation = EXCLUDED.manual_activation,
                neg_risk_other = EXCLUDED.neg_risk_other,
                uma_resolution_statuses = EXCLUDED.uma_resolution_statuses,
                pending_deployment = EXCLUDED.pending_deployment,
                deploying = EXCLUDED.deploying,
                rfq_enabled = EXCLUDED.rfq_enabled,
                holding_rewards_enabled = EXCLUDED.holding_rewards_enabled,
                fees_enabled = EXCLUDED.fees_enabled,
                received_at = CURRENT_TIMESTAMP",
            value_strings.join(", ")
        );

        client.execute(&query, &params).await?;
        total_inserted += chunk.len();
        
        info!(chunk_size = chunk.len(), total = total_inserted, "Inserted market_gamma chunk");
    }

    info!(count = total_inserted, "Completed inserting market_gamma batch");

    Ok(())
}