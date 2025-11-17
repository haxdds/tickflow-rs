//! PostgreSQL handler for YahooMessage.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::connectors::yahoo::types::YahooMessage;
use crate::storage::postgres::DatabaseMessageHandler;
use anyhow::Result;
use paft_domain::period::Period;
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
            // Adjust schema based on actual IncomeStatementRow, BalanceSheetRow, Calendar structures
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
                        let total_revenue = match row.inner.total_revenue {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue, // Skip this message if the value is missing
                        };
                        let gross_profit = match row.inner.gross_profit {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let operating_income = match row.inner.operating_income {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let net_income = match row.inner.net_income {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };

                        let period_date = match &row.inner.period {
                            Period::Date(date) => {
                                // Assuming date is already a NaiveDate or can be converted to one
                                *date // or date.clone() if needed
                            }
                            _ => continue,
                        };

                        if let Err(e) = client.execute(
                            "INSERT INTO quarterly_income_statements 
                                (symbol, period_date, total_revenue, gross_profit, operating_income, net_income)
                            VALUES ($1, $2, $3, $4, $5, $6)
                            ON CONFLICT (symbol, period_date) DO NOTHING",
                            &[
                                &row.symbol,
                                &period_date,  // Pass as string, PostgreSQL will parse it
                                &total_revenue,
                                &gross_profit,
                                &operating_income,
                                &net_income
                            ]
                        ).await {
                            tracing::error!(
                                "Failed to insert income_statement for symbol: {}, period_date: {}, total_revenue: {}, gross_profit: {}, operating_income: {}, net_income: {}. Error: {}",
                                &row.symbol,
                                period_date,
                                total_revenue,
                                gross_profit,
                                operating_income,
                                net_income,
                                e
                            );
                        }
                    }
                    YahooMessage::BalanceSheet(row) => {
                        let total_assets = match row.inner.total_assets {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let total_liabilities = match row.inner.total_liabilities {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let total_equity = match row.inner.total_equity {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let cash = match row.inner.cash {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let long_term_debt = match row.inner.long_term_debt {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let shares_outstanding = match row.inner.shares_outstanding {
                            Some(val) => val,
                            None => continue,
                        };

                        let period_date = match &row.inner.period {
                            Period::Date(date) => *date,
                            _ => continue,
                        };

                        if let Err(e) = client.execute(
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
                                &(shares_outstanding as i64),
                            ]
                        ).await {
                            tracing::error!(
                                "Failed to insert balance_sheet for symbol: {}, period_date: {}, total_assets: {}, total_liabilities: {}, total_equity: {}, cash: {}, long_term_debt: {}, shares_outstanding: {}. Error: {}",
                                &row.symbol,
                                period_date,
                                total_assets,
                                total_liabilities,
                                total_equity,
                                cash,
                                long_term_debt,
                                shares_outstanding,
                                e
                            );
                        }
                    }
                    YahooMessage::Cashflow(row) => {
                        let operating_cashflow = match row.inner.operating_cashflow {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let capital_expenditures = match row.inner.capital_expenditures {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let free_cash_flow = match row.inner.free_cash_flow {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };
                        let net_income = match row.inner.net_income {
                            Some(val) => val
                                .amount()
                                .to_f64()
                                .expect("Could not convert Decimal to f64"),
                            None => continue,
                        };

                        let period_date = match &row.inner.period {
                            Period::Date(date) => *date,
                            _ => continue,
                        };

                        if let Err(e) = client.execute(
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
                                &net_income
                            ]
                        ).await {
                            tracing::error!(
                                "Failed to insert cashflow for symbol: {}, period_date: {}, operating_cashflow: {}, capital_expenditures: {}, free_cash_flow: {}, net_income: {}. Error: {}",
                                &row.symbol,
                                period_date,
                                operating_cashflow,
                                capital_expenditures,
                                free_cash_flow,
                                net_income,
                                e
                            );
                        }
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
