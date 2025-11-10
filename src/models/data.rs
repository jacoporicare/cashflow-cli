use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Top-level data structure persisted in RON format
/// This is the source of truth stored in ~/.cashflow/data.ron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashflowData {
    pub recurring: Vec<RecurringTransaction>,
    pub one_time: Vec<OneTimeTransaction>,
    pub balance_snapshots: Vec<BalanceSnapshot>,
}

impl Default for CashflowData {
    fn default() -> Self {
        Self {
            recurring: Vec::new(),
            one_time: Vec::new(),
            balance_snapshots: Vec::new(),
        }
    }
}

/// Recurring transaction template that generates monthly transactions
/// Example: Netflix subscription on 14th of each month
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringTransaction {
    pub id: Uuid,
    pub description: String,
    /// Positive for income, negative for expenses
    pub amount: Decimal,
    /// Day of month when payment occurs (1-31)
    pub day_of_month: u8,
    /// Inactive transactions are not projected but kept for history
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl RecurringTransaction {
    pub fn new(description: String, amount: Decimal, day_of_month: u8) -> Self {
        Self {
            id: Uuid::new_v4(),
            description,
            amount,
            day_of_month,
            active: true,
            created_at: Utc::now(),
        }
    }
}

/// One-time transaction (e.g., "Převod z Air Bank")
/// These are not recurring and should be pruned after they're in the past
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTimeTransaction {
    pub id: Uuid,
    pub description: String,
    pub amount: Decimal,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl OneTimeTransaction {
    pub fn new(description: String, amount: Decimal, date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            description,
            amount,
            date,
            created_at: Utc::now(),
        }
    }
}

/// Balance snapshot at a specific date
/// Used as starting point for cashflow projections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub id: Uuid,
    pub date: NaiveDate,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
}

impl BalanceSnapshot {
    pub fn new(date: NaiveDate, balance: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            date,
            balance,
            created_at: Utc::now(),
        }
    }
}

/// Transaction view for display (not persisted)
/// Generated from recurring templates or one-time transactions
#[derive(Debug, Clone)]
pub struct TransactionView {
    pub date: NaiveDate,
    pub day_of_month: u8,
    pub description: String,
    pub amount: Decimal,
    /// True if from one_time, false if from recurring template
    pub is_one_time: bool,
    /// Calculated running balance after this transaction
    pub balance_after: Decimal,
}

impl TransactionView {
    pub fn from_recurring(
        txn: &RecurringTransaction,
        date: NaiveDate,
        balance_after: Decimal,
    ) -> Self {
        Self {
            date,
            day_of_month: txn.day_of_month,
            description: txn.description.clone(),
            amount: txn.amount,
            is_one_time: false,
            balance_after,
        }
    }

    pub fn from_one_time(txn: &OneTimeTransaction, balance_after: Decimal) -> Self {
        Self {
            date: txn.date,
            day_of_month: txn.date.day() as u8,
            description: txn.description.clone(),
            amount: txn.amount,
            is_one_time: true,
            balance_after,
        }
    }
}
