use crate::Result;
use crate::cli::{format_amount, print_plan_table};
use crate::projection::project_cashflow;
use crate::storage::load_data;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Executes the plan command: shows cashflow projection for N days
pub async fn execute_plan(days: i64) -> Result<()> {
    // Load data from RON file
    let data = load_data().await?;

    // Check if we have a balance snapshot
    if data.balance_snapshots.is_empty() {
        anyhow::bail!(
            "No balance snapshot found. Please set your current balance first:\n  \
             cashflow balance set <amount>"
        );
    }

    // Generate projection - returns (starting_balance, today, projected_transactions)
    let (starting_balance, today, projected) = project_cashflow(&data, days)?;

    if projected.is_empty() {
        println!("No transactions scheduled for the next {} days.", days);
        println!("Current balance: {}", format_amount(starting_balance));
        return Ok(());
    }

    // Warning threshold: 10,000 Kč
    let warning_threshold = Decimal::from_str("10000").unwrap();

    // Print the table with today's date and calculated balance
    print_plan_table(&projected, starting_balance, today, warning_threshold);

    Ok(())
}
