use crate::Result;
use crate::cli::format_amount;
use crate::models::BalanceSnapshot;
use crate::storage::{load_data, save_data};
use anyhow::Context;
use chrono::{Local, NaiveDate};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Executes the balance set command
pub async fn execute_balance_set(amount_str: &str, date_str: Option<&str>) -> Result<()> {
    // Parse amount
    let amount = parse_amount(amount_str)?;

    // Parse date (default to today)
    let date = if let Some(date_str) = date_str {
        parse_date(date_str)?
    } else {
        Local::now().date_naive()
    };

    // Load data
    let mut data = load_data().await?;

    // Check if snapshot for this date already exists
    if let Some(existing) = data.balance_snapshots.iter_mut().find(|s| s.date == date) {
        // Update existing snapshot
        existing.balance = amount;
        println!(
            "Updated balance for {}: {}",
            date.format("%d.%m.%Y"),
            format_amount(amount)
        );
    } else {
        // Create new snapshot
        let snapshot = BalanceSnapshot::new(date, amount);
        data.balance_snapshots.push(snapshot);
        println!(
            "Set balance for {}: {}",
            date.format("%d.%m.%Y"),
            format_amount(amount)
        );
    }

    // Save data
    save_data(&data).await?;

    Ok(())
}

/// Executes the balance show command
pub async fn execute_balance_show() -> Result<()> {
    let data = load_data().await?;

    if data.balance_snapshots.is_empty() {
        println!("No balance snapshots found.");
        println!("Set your current balance first:");
        println!("  cashflow balance set <amount>");
        return Ok(());
    }

    // Find the most recent snapshot
    let latest = data
        .balance_snapshots
        .iter()
        .max_by_key(|s| s.date)
        .unwrap();

    println!(
        "Balance on {}: {}",
        latest.date.format("%d.%m.%Y"),
        format_amount(latest.balance)
    );

    Ok(())
}

/// Parses amount from string, supports both formats:
/// - "22158" or "22 158" (without sign, positive)
/// - "-478" or "- 478" (negative)
pub fn parse_amount(s: &str) -> Result<Decimal> {
    // Remove spaces
    let cleaned = s.replace(' ', "");

    Decimal::from_str(&cleaned).context("Invalid amount format. Use: 22158 or -478")
}

/// Parses date from string, supports formats:
/// - "DD.MM.YYYY" (Czech format)
/// - "YYYY-MM-DD" (ISO format)
pub fn parse_date(s: &str) -> Result<NaiveDate> {
    // Try Czech format first: DD.MM.YYYY
    if let Ok(date) = NaiveDate::parse_from_str(s, "%d.%m.%Y") {
        return Ok(date);
    }

    // Try ISO format: YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date);
    }

    anyhow::bail!("Invalid date format. Use: DD.MM.YYYY or YYYY-MM-DD")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_amount() {
        assert_eq!(
            parse_amount("22158").unwrap(),
            Decimal::from_str("22158").unwrap()
        );
        assert_eq!(
            parse_amount("22 158").unwrap(),
            Decimal::from_str("22158").unwrap()
        );
        assert_eq!(
            parse_amount("-478").unwrap(),
            Decimal::from_str("-478").unwrap()
        );
        assert_eq!(
            parse_amount("- 478").unwrap(),
            Decimal::from_str("-478").unwrap()
        );
    }

    #[test]
    fn test_parse_date() {
        let expected = NaiveDate::from_ymd_opt(2025, 10, 13).unwrap();

        assert_eq!(parse_date("13.10.2025").unwrap(), expected);
        assert_eq!(parse_date("2025-10-13").unwrap(), expected);
    }
}
