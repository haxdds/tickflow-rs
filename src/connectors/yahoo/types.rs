use crate::core::{Message};
use serde::{Deserialize};
use yfinance_rs::fundamentals::{Calendar, IncomeStatementRow, BalanceSheetRow};
use std::fmt::Display;

#[derive(Deserialize, Clone)]
pub enum YahooMessage {
    Calendar(Calendar),
    IncomeStatement(IncomeStatementRow),
    BalanceSheet(BalanceSheetRow),
    // Cashflow(Vec<Cashflow>)
}

impl Message for YahooMessage {}

impl Display for YahooMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YahooMessage::Calendar(cal) => write!(f, "Calendar: {:?}", cal),
            YahooMessage::IncomeStatement(row) => write!(f, "IncomeStatement: {:?}", row),
            YahooMessage::BalanceSheet(row) => write!(f, "BalanceSheet: {:?}", row),
        }
    }
}
