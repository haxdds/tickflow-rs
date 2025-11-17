//! PostgreSQL handler for YahooMessage.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::connectors::yahoo::types::YahooMessage;
use crate::storage::postgres::DatabaseMessageHandler;
use anyhow::Result;
use paft_domain::period::Period;
use paft_money::money::Money;
use rust_decimal::prelude::ToPrimitive;
use tokio_postgres::Client;

pub struct YahooMessageHandler;

impl DatabaseMessageHandler<YahooMessage> for YahooMessageHandler {
    fn initialize_schema(
        &self,
        client: Arc<Client>,
    ) -> Pin<Box<dyn Future<Output = Result<(), tokio_postgres::Error>> + Send>> {
        Box::pin(async move {
            // Create tables for Yahoo finance data
            client
                .execute(
                    "CREATE TABLE IF NOT EXISTS quarterly_income_statements (
                        id SERIAL PRIMARY KEY,
                        symbol VARCHAR(10) NOT NULL,
                        period_date DATE,                     
                        total_revenue DOUBLE PRECISION,
                        gross_profit DOUBLE PRECISION,
                        operating_income DOUBLE PRECISION,
                        net_income DOUBLE PRECISION,
                        received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        UNIQUE(symbol, period_date)
                    )",
                    &[],
                )
                .await?;

            client
                .execute(
                    "CREATE TABLE IF NOT EXISTS quarterly_balance_sheets (
                        id SERIAL PRIMARY KEY,
                        symbol VARCHAR(10) NOT NULL,
                        period_date DATE,                     
                        total_assets DOUBLE PRECISION,
                        total_liabilities DOUBLE PRECISION,
                        total_equity DOUBLE PRECISION,
                        cash DOUBLE PRECISION,
                        long_term_debt DOUBLE PRECISION,
                        shares_outstanding BIGINT,
                        received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        UNIQUE(symbol, period_date)
                    )",
                    &[],
                )
                .await?;

            client
                .execute(
                    "CREATE TABLE IF NOT EXISTS quarterly_cashflow_statements (
                        id SERIAL PRIMARY KEY,
                        symbol VARCHAR(10) NOT NULL,
                        period_date DATE,                     
                        operating_cashflow DOUBLE PRECISION,
                        capital_expenditures DOUBLE PRECISION,
                        free_cash_flow DOUBLE PRECISION,
                        net_income DOUBLE PRECISION,
                        received_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        UNIQUE(symbol, period_date)
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
        batch: Vec<YahooMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            for message in batch {
                match message {
                    YahooMessage::IncomeStatement(row) => {
                        Self::insert_income_statement(&client, row).await;
                    }
                    YahooMessage::BalanceSheet(row) => {
                        Self::insert_balance_sheet(&client, row).await;
                    }
                    YahooMessage::Cashflow(row) => {
                        Self::insert_cashflow(&client, row).await;
                    }
                    YahooMessage::Calendar(_cal) => {
                        // Calendar is not handled for now.
                    }
                }
            }
            Ok(())
        })
    }
}

impl YahooMessageHandler {
    /// Extract a Decimal amount to f64, returning None if missing
    fn extract_amount(amount: Option<&Money>) -> Option<f64> {
        amount?.amount().to_f64()
    }

    /// Extract period date from Period enum
    fn extract_period_date(period: &Period) -> Option<chrono::NaiveDate> {
        match period {
            Period::Date(date) => Some(*date),
            _ => None,
        }
    }

    async fn insert_income_statement(
        client: &Client,
        row: crate::connectors::yahoo::types::IncomeStatementRow,
    ) {
        let period_date = match Self::extract_period_date(&row.inner.period) {
            Some(date) => date,
            None => {
                tracing::warn!(
                    "Skipping income statement for {}: invalid period",
                    row.symbol
                );
                return;
            }
        };

        let total_revenue = match Self::extract_amount(row.inner.total_revenue.as_ref()) {
            Some(val) => val,
            None => {
                tracing::warn!(
                    "Skipping income statement for {}: missing total_revenue",
                    row.symbol
                );
                return;
            }
        };

        let gross_profit = Self::extract_amount(row.inner.gross_profit.as_ref());
        let operating_income = Self::extract_amount(row.inner.operating_income.as_ref());
        let net_income = Self::extract_amount(row.inner.net_income.as_ref());

        if let Err(e) = client
            .execute(
                "INSERT INTO quarterly_income_statements 
                    (symbol, period_date, total_revenue, gross_profit, operating_income, net_income)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (symbol, period_date) DO NOTHING",
                &[
                    &row.symbol,
                    &period_date,
                    &total_revenue,
                    &gross_profit,
                    &operating_income,
                    &net_income,
                ],
            )
            .await
        {
            tracing::error!(
                "Failed to insert income statement for {} ({}): {}",
                row.symbol,
                period_date,
                e
            );
        }
    }

    async fn insert_balance_sheet(
        client: &Client,
        row: crate::connectors::yahoo::types::BalanceSheetRow,
    ) {
        let period_date = match Self::extract_period_date(&row.inner.period) {
            Some(date) => date,
            None => {
                tracing::warn!("Skipping balance sheet for {}: invalid period", row.symbol);
                return;
            }
        };

        let total_assets = match Self::extract_amount(row.inner.total_assets.as_ref()) {
            Some(val) => val,
            None => {
                tracing::warn!(
                    "Skipping balance sheet for {}: missing total_assets",
                    row.symbol
                );
                return;
            }
        };

        let total_liabilities = Self::extract_amount(row.inner.total_liabilities.as_ref());
        let total_equity = Self::extract_amount(row.inner.total_equity.as_ref());
        let cash = Self::extract_amount(row.inner.cash.as_ref());
        let long_term_debt = Self::extract_amount(row.inner.long_term_debt.as_ref());
        let shares_outstanding = row.inner.shares_outstanding;

        if let Err(e) = client
            .execute(
                "INSERT INTO quarterly_balance_sheets 
                    (symbol, period_date, total_assets, total_liabilities, total_equity, cash, long_term_debt, shares_outstanding)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (symbol, period_date) DO NOTHING",
                &[
                    &row.symbol,
                    &period_date,
                    &total_assets,
                    &total_liabilities,
                    &total_equity,
                    &cash,
                    &long_term_debt,
                    &(shares_outstanding.map(|v| v as i64)),
                ],
            )
            .await
        {
            tracing::error!(
                "Failed to insert balance sheet for {} ({}): {}",
                row.symbol,
                period_date,
                e
            );
        }
    }

    async fn insert_cashflow(client: &Client, row: crate::connectors::yahoo::types::CashflowRow) {
        let period_date = match Self::extract_period_date(&row.inner.period) {
            Some(date) => date,
            None => {
                tracing::warn!("Skipping cashflow for {}: invalid period", row.symbol);
                return;
            }
        };

        let operating_cashflow = match Self::extract_amount(row.inner.operating_cashflow.as_ref()) {
            Some(val) => val,
            None => {
                tracing::warn!(
                    "Skipping cashflow for {}: missing operating_cashflow",
                    row.symbol
                );
                return;
            }
        };

        let capital_expenditures = Self::extract_amount(row.inner.capital_expenditures.as_ref());
        let free_cash_flow = Self::extract_amount(row.inner.free_cash_flow.as_ref());
        let net_income = Self::extract_amount(row.inner.net_income.as_ref());

        if let Err(e) = client
            .execute(
                "INSERT INTO quarterly_cashflow_statements 
                    (symbol, period_date, operating_cashflow, capital_expenditures, free_cash_flow, net_income)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (symbol, period_date) DO NOTHING",
                &[
                    &row.symbol,
                    &period_date,
                    &operating_cashflow,
                    &capital_expenditures,
                    &free_cash_flow,
                    &net_income,
                ],
            )
            .await
        {
            tracing::error!(
                "Failed to insert cashflow for {} ({}): {}",
                row.symbol,
                period_date,
                e
            );
        }
    }
}
