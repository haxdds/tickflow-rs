use crate::core::Message;
use chrono::NaiveDateTime;
use serde::Deserialize;
use yfinance_rs::fundamentals::{
    BalanceSheetRow as YBalanceSheetRow, CashflowRow as YCashflowRow,
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
pub struct CalendarEntry {
    pub symbol: String,
    pub date_type: CalendarDateType,
    pub date: NaiveDateTime,
}

#[derive(Deserialize, Clone, Debug)]
pub enum CalendarDateType {
    #[serde(rename = "earnings")]
    Earnings,
    #[serde(rename = "ex_dividend")]
    ExDividend,
    #[serde(rename = "dividend_payment")]
    DividendPayment,
}

#[derive(Deserialize, Clone, Debug)]
pub enum YahooMessage {
    Calendar(CalendarEntry),
    IncomeStatement(IncomeStatementRow),
    BalanceSheet(BalanceSheetRow),
    Cashflow(CashflowRow),
}

impl Message for YahooMessage {}
