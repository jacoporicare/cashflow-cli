use crate::Result;
use crate::cli::{PlanDisplayOptions, format_amount, print_plan_table};
use crate::projection::project_cashflow;
use crate::storage::load_data;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Executes the plan command: shows cashflow projection for N days
pub async fn execute_plan(days: i64, show_past: bool) -> Result<()> {
    // Load data from RON file
    let data = load_data().await?;

    // Check if we have a balance snapshot
    if data.balance_snapshots.is_empty() {
        anyhow::bail!(
            "No balance snapshot found. Please set your current balance first:\n  \
             cashflow balance set <amount>"
        );
    }

    // Generate projection
    let projection = project_cashflow(&data, days)?;

    if projection.future_txns.is_empty() {
        println!("No transactions scheduled for the next {} days.", days);
        println!(
            "Current balance: {}",
            format_amount(projection.starting_balance)
        );
        return Ok(());
    }

    // Warning threshold: 10,000 Kč
    let warning_threshold = Decimal::from_str("10000").unwrap();

    // Print the table with today's date and calculated balance
    let display_options = PlanDisplayOptions {
        warning_threshold,
        show_past,
    };
    print_plan_table(&projection, &display_options);

    Ok(())
}
