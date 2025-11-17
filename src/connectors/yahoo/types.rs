use crate::core::{Message};
use serde::{Deserialize};
use yfinance_rs::fundamentals::{Calendar, IncomeStatementRow as YIncomeStatementRow, BalanceSheetRow as YBalanceSheetRow, CashflowRow as YCashflowRow};
use std::fmt::Display;

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

/// Optionally, you can also wrap Calendar, but if not required, don't.
/// If you do:
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

impl Display for YahooMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YahooMessage::Calendar(cal) => write!(f, "Calendar: {:?}", cal),
            YahooMessage::IncomeStatement(row) => write!(f, "IncomeStatement: {:?}", row),
            YahooMessage::BalanceSheet(row) => write!(f, "BalanceSheet: {:?}", row),
            YahooMessage::Cashflow(row) => write!(f, "Cashflow: {:?}", row),
        }
    }
}
