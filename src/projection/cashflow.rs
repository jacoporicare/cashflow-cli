use crate::models::{
    BalanceSnapshot, CashflowData, OneTimeTransaction, ProjectedTransaction, RecurringTransaction,
};
use anyhow::anyhow;
use chrono::{Datelike, Duration, Local, NaiveDate};

/// Generates cashflow projection for the next N days from today
/// Returns: (starting_balance, today, projected_transactions)
pub fn project_cashflow(
    data: &CashflowData,
    days: i64,
) -> anyhow::Result<(rust_decimal::Decimal, NaiveDate, Vec<ProjectedTransaction>)> {
    // Find the most recent balance snapshot
    let snapshot = find_latest_balance_snapshot(data)?;
    let today = Local::now().date_naive();

    // Step 1: Calculate current balance as of today
    // Start with snapshot balance and apply all transactions from snapshot date to today (exclusive)
    let mut current_balance = snapshot.balance;

    if today > snapshot.date {
        // Calculate balance from snapshot date to today
        let balance_calc_end = today;
        let mut past_transactions = Vec::new();

        // Generate recurring transactions from snapshot to today
        for recurring in &data.recurring {
            if !recurring.active {
                continue;
            }
            let recurring_txns =
                generate_recurring_transactions(recurring, snapshot.date, balance_calc_end);
            past_transactions.extend(recurring_txns);
        }

        // Add one-time transactions from snapshot to today
        for one_time in &data.one_time {
            if one_time.date > snapshot.date && one_time.date < today {
                past_transactions.push((one_time.date, one_time.clone(), false));
            }
        }

        // Sort by date and apply to current balance
        past_transactions.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.created_at.cmp(&b.1.created_at))
        });

        for (_date, txn, _is_recurring) in past_transactions {
            current_balance += txn.amount;
        }
    }

    // Step 2: Generate projection from today to today + days
    let end_date = today + Duration::days(days);
    let mut transactions = Vec::new();

    // Generate recurring transactions from today onwards
    for recurring in &data.recurring {
        if !recurring.active {
            continue;
        }

        let recurring_txns = generate_recurring_transactions(recurring, today, end_date);
        transactions.extend(recurring_txns);
    }

    // Add one-time transactions from today onwards
    for one_time in &data.one_time {
        if one_time.date >= today && one_time.date <= end_date {
            transactions.push((one_time.date, one_time.clone(), false));
        }
    }

    // Sort by date, then by creation time (deterministic ordering)
    transactions.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.created_at.cmp(&b.1.created_at))
    });

    // Step 3: Calculate running balance and create projected transactions
    let starting_balance = current_balance; // Save the starting balance for today
    let mut projected = Vec::new();

    for (date, txn, is_recurring) in transactions {
        current_balance += txn.amount;

        let projected_txn = if is_recurring {
            // It's a recurring transaction
            // Match by id to handle multiple recurring transactions with same description
            if let Some(recurring) = data.recurring.iter().find(|r| r.id == txn.id) {
                ProjectedTransaction::from_recurring(recurring, date, current_balance)
            } else {
                continue;
            }
        } else {
            // It's a one-time transaction
            ProjectedTransaction::from_one_time(&txn, current_balance)
        };

        projected.push(projected_txn);
    }

    Ok((starting_balance, today, projected))
}

/// Finds the most recent balance snapshot
fn find_latest_balance_snapshot(data: &CashflowData) -> anyhow::Result<&BalanceSnapshot> {
    data.balance_snapshots
        .iter()
        .max_by_key(|s| s.date)
        .ok_or_else(|| anyhow!("No balance snapshots found. Please set initial balance first."))
}

/// Generates recurring transaction instances for the projection window
/// Handles month boundaries (e.g., day 31 in February becomes last day of month)
fn generate_recurring_transactions(
    recurring: &RecurringTransaction,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Vec<(NaiveDate, OneTimeTransaction, bool)> {
    let mut transactions = Vec::new();
    let mut current_date = start_date;

    // Loop through months until we've passed end_date's month
    loop {
        // Calculate the target date in current month
        if let Some(txn_date) = get_transaction_date_in_month(current_date, recurring.day_of_month)
        {
            // Only include if it's after start_date and within end_date
            if txn_date > start_date && txn_date <= end_date {
                // Convert to OneTimeTransaction for uniform processing
                let one_time = OneTimeTransaction {
                    id: recurring.id,
                    description: recurring.description.clone(),
                    amount: recurring.amount,
                    date: txn_date,
                    created_at: recurring.created_at,
                };

                transactions.push((txn_date, one_time, true));
            }
        }

        // Move to next month
        current_date = next_month(current_date);

        // Exit if we've moved past end_date's month
        // Compare year and month to ensure we don't skip the end_date's month
        if current_date.year() > end_date.year()
            || (current_date.year() == end_date.year() && current_date.month() > end_date.month())
        {
            break;
        }
    }

    transactions
}

/// Gets the actual date for a recurring transaction in a given month
/// Handles edge cases like day 31 in months with fewer days
fn get_transaction_date_in_month(base_date: NaiveDate, day_of_month: u8) -> Option<NaiveDate> {
    let year = base_date.year();
    let month = base_date.month();

    // Handle day_of_month that doesn't exist in this month
    // Example: day 31 in February → last day of February
    let days_in_month = days_in_month(year, month);
    let actual_day = day_of_month.min(days_in_month);

    NaiveDate::from_ymd_opt(year, month, actual_day as u32)
}

/// Returns the number of days in a given month
fn days_in_month(year: i32, month: u32) -> u8 {
    // Last day of month is first day of next month minus 1 day
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };

    let last_day = next_month_first - Duration::days(1);
    last_day.day() as u8
}

/// Advances date to the same day next month
fn next_month(date: NaiveDate) -> NaiveDate {
    let year = date.year();
    let month = date.month();
    let day = date.day();

    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, day).unwrap_or_else(|| {
            // If day doesn't exist in January (impossible for Jan), fallback
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        })
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, day).unwrap_or_else(|| {
            // If day doesn't exist in next month, use first day of next month
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RecurringTransaction;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2025, 1), 31); // January
        assert_eq!(days_in_month(2025, 2), 28); // February (not leap)
        assert_eq!(days_in_month(2024, 2), 29); // February (leap year)
        assert_eq!(days_in_month(2025, 4), 30); // April
    }

    #[test]
    fn test_get_transaction_date_in_month() {
        let base = NaiveDate::from_ymd_opt(2025, 2, 15).unwrap();

        // Day 14 in February → Feb 14
        assert_eq!(
            get_transaction_date_in_month(base, 14),
            NaiveDate::from_ymd_opt(2025, 2, 14)
        );

        // Day 31 in February → Feb 28
        assert_eq!(
            get_transaction_date_in_month(base, 31),
            NaiveDate::from_ymd_opt(2025, 2, 28)
        );
    }

    #[test]
    fn test_projection_with_recurring() {
        let mut data = CashflowData::default();
        let today = Local::now().date_naive();

        // Set balance snapshot to yesterday
        let yesterday = today - Duration::days(1);
        data.balance_snapshots.push(BalanceSnapshot::new(
            yesterday,
            Decimal::from_str("22158").unwrap(),
        ));

        // Add Netflix on today's day of month
        data.recurring.push(RecurringTransaction::new(
            "Netflix".to_string(),
            Decimal::from_str("-478").unwrap(),
            today.day() as u8,
        ));

        // Project 30 days from today
        let (_starting_balance, projection_date, projected) = project_cashflow(&data, 30).unwrap();

        // Verify projection starts from today
        assert_eq!(projection_date, today);

        // Should have at least Netflix transaction
        assert!(!projected.is_empty());
        let netflix = projected.iter().find(|p| p.description == "Netflix");
        assert!(netflix.is_some());

        let netflix = netflix.unwrap();
        assert_eq!(netflix.date.day(), today.day());
        assert_eq!(netflix.amount, Decimal::from_str("-478").unwrap());
    }

    #[test]
    fn test_projection_with_duplicate_descriptions() {
        let mut data = CashflowData::default();
        let today = Local::now().date_naive();

        // Set balance snapshot to a week ago
        let week_ago = today - Duration::days(7);
        data.balance_snapshots.push(BalanceSnapshot::new(
            week_ago,
            Decimal::from_str("10000").unwrap(),
        ));

        // Add first "Služby" on day 1 with amount -2500
        data.recurring.push(RecurringTransaction::new(
            "Služby".to_string(),
            Decimal::from_str("-2500").unwrap(),
            1,
        ));

        // Add second "Služby" on day 20 with amount -2940
        data.recurring.push(RecurringTransaction::new(
            "Služby".to_string(),
            Decimal::from_str("-2940").unwrap(),
            20,
        ));

        // Project 60 days to ensure we catch at least one occurrence of each
        let (_starting_balance, projection_date, projected) = project_cashflow(&data, 60).unwrap();

        // Verify projection starts from today
        assert_eq!(projection_date, today);

        // Should have both "Služby" transactions (might be in current or next month)
        let sluzby_transactions: Vec<_> = projected
            .iter()
            .filter(|p| p.description == "Služby")
            .collect();

        // We should have at least one of each type within 60 days
        assert!(
            sluzby_transactions.len() >= 2,
            "Should have at least 2 Služby transactions"
        );

        // Verify we have the transaction with amount -2500
        let sluzby1_exists = sluzby_transactions
            .iter()
            .any(|p| p.day_of_month == 1 && p.amount == Decimal::from_str("-2500").unwrap());
        assert!(
            sluzby1_exists,
            "Should have Služby transaction on day 1 with amount -2500"
        );

        // Verify we have the transaction with amount -2940
        let sluzby2_exists = sluzby_transactions
            .iter()
            .any(|p| p.day_of_month == 20 && p.amount == Decimal::from_str("-2940").unwrap());
        assert!(
            sluzby2_exists,
            "Should have Služby transaction on day 20 with amount -2940"
        );
    }
}
