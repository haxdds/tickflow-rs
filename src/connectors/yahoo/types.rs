use crate::core::Message;
use serde::Deserialize;
use yfinance_rs::fundamentals::{
    BalanceSheetRow as YBalanceSheetRow, Calendar, CashflowRow as YCashflowRow,
    IncomeStatementRow as YIncomeStatementRow,
};

/// Wraps an IncomeStatementRow with a symbol field.
#[derive(Deserialize, Clone, Debug)]
pub struct IncomeStatementRow {
    pub symbol: String,
    #[serde(flatten)]
    pub inner: YIncomeStatementRow,
}

/// Wraps a BalanceSheetRow with a symbol field.
#[derive(Deserialize, Clone, Debug)]
pub struct BalanceSheetRow {
    pub symbol: String,
    #[serde(flatten)]
    pub inner: YBalanceSheetRow,
}

/// Wraps a CashflowRow with a symbol field.
#[derive(Deserialize, Clone, Debug)]
pub struct CashflowRow {
    pub symbol: String,
    #[serde(flatten)]
    pub inner: YCashflowRow,
}

/// Wraps a Calendar with a symbol field.
#[derive(Deserialize, Clone, Debug)]
pub struct CalendarRow {
    pub symbol: String,
    #[serde(flatten)]
    pub inner: Calendar,
}

#[derive(Deserialize, Clone, Debug)]
pub enum YahooMessage {
    Calendar(CalendarRow),
    IncomeStatement(IncomeStatementRow),
    BalanceSheet(BalanceSheetRow),
    Cashflow(CashflowRow),
}

impl Message for YahooMessage {}
