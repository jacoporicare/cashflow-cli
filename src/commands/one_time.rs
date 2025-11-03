use crate::Result;
use crate::cli::format_amount;
use crate::commands::balance::{parse_amount, parse_date};
use crate::models::OneTimeTransaction;
use crate::storage::{load_data, save_data};
use anyhow::Context;
use chrono::Local;
use comfy_table::{Attribute, Cell, Color, Table, presets::UTF8_FULL};
use uuid::Uuid;

/// Executes add command (one-time transaction)
pub async fn execute_one_time_add(
    description: &str,
    amount_str: &str,
    date_str: &str,
) -> Result<()> {
    // Parse amount and date
    let amount = parse_amount(amount_str)?;
    let date = parse_date(date_str)?;

    // Load data
    let mut data = load_data().await?;

    // Create one-time transaction
    let transaction = OneTimeTransaction::new(description.to_string(), amount, date);

    data.one_time.push(transaction.clone());

    // Save data
    save_data(&data).await?;

    println!("Added one-time transaction:");
    println!("  Description: {}", transaction.description);
    println!("  Amount: {}", format_amount(transaction.amount));
    println!("  Date: {}", transaction.date.format("%d.%m.%Y"));
    println!("  ID: {}", transaction.id);

    Ok(())
}

/// Executes list command (one-time transactions)
pub async fn execute_one_time_list(upcoming: bool) -> Result<()> {
    let data = load_data().await?;

    let mut transactions = data.one_time.clone();

    // Filter upcoming if requested
    if upcoming {
        let today = Local::now().date_naive();
        transactions.retain(|t| t.date >= today);
    }

    if transactions.is_empty() {
        if upcoming {
            println!("No upcoming one-time transactions found.");
        } else {
            println!("No one-time transactions found.");
        }
        println!("Add one with:");
        println!("  cashflow one-time add -d <description> -a <amount> --date <date>");
        return Ok(());
    }

    // Sort by date
    transactions.sort_by_key(|t| t.date);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold),
        Cell::new("Description").add_attribute(Attribute::Bold),
        Cell::new("Amount").add_attribute(Attribute::Bold),
        Cell::new("Date").add_attribute(Attribute::Bold),
    ]);

    for txn in &transactions {
        let id_short = txn.id.to_string().chars().take(8).collect::<String>();
        let amount_color = if txn.amount.is_sign_negative() {
            Color::Red
        } else {
            Color::Green
        };

        table.add_row(vec![
            Cell::new(id_short),
            Cell::new(&txn.description),
            Cell::new(format_amount(txn.amount)).fg(amount_color),
            Cell::new(txn.date.format("%d.%m.%Y").to_string()),
        ]);
    }

    println!("{table}");
    println!();
    println!("Total: {} one-time transactions", transactions.len());

    Ok(())
}

/// Executes one-time edit command
pub async fn execute_one_time_edit(
    id_str: &str,
    amount: Option<&str>,
    date: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    let mut data = load_data().await?;
    let id = parse_uuid(id_str, &data.one_time)?;

    let transaction = data
        .one_time
        .iter_mut()
        .find(|t| t.id == id)
        .context("One-time transaction not found")?;

    if let Some(amount_str) = amount {
        transaction.amount = parse_amount(amount_str)?;
        println!("Updated amount: {}", format_amount(transaction.amount));
    }

    if let Some(date_str) = date {
        transaction.date = parse_date(date_str)?;
        println!("Updated date: {}", transaction.date.format("%d.%m.%Y"));
    }

    if let Some(desc) = description {
        transaction.description = desc.to_string();
        println!("Updated description: {}", desc);
    }

    save_data(&data).await?;
    println!("One-time transaction updated successfully.");

    Ok(())
}

/// Executes one-time delete command
pub async fn execute_one_time_delete(id_str: &str) -> Result<()> {
    let mut data = load_data().await?;
    let id = parse_uuid(id_str, &data.one_time)?;

    let index = data
        .one_time
        .iter()
        .position(|t| t.id == id)
        .context("One-time transaction not found")?;

    let removed = data.one_time.remove(index);
    save_data(&data).await?;

    println!("Deleted one-time transaction: {}", removed.description);

    Ok(())
}

/// Parses UUID from string, supports both full and short (8 char) format
fn parse_uuid(s: &str, transactions: &[OneTimeTransaction]) -> Result<Uuid> {
    // Try full UUID first
    if let Ok(uuid) = Uuid::parse_str(s) {
        return Ok(uuid);
    }

    // Try short format (8 chars) - match against existing UUIDs by prefix
    if s.len() >= 8 {
        let prefix = s.to_lowercase();
        let matching: Vec<_> = transactions
            .iter()
            .filter(|t| {
                let uuid_str = t.id.to_string().to_lowercase();
                uuid_str.starts_with(&prefix)
            })
            .collect();

        match matching.len() {
            0 => {
                anyhow::bail!(
                    "No one-time transaction found with ID starting with '{}'. Use 'one-time list' to see available IDs.",
                    s
                )
            }
            1 => return Ok(matching[0].id),
            _ => {
                anyhow::bail!(
                    "Multiple transactions match '{}'. Use full UUID to be more specific. Use 'one-time list' to see all IDs.",
                    s
                )
            }
        }
    }

    anyhow::bail!(
        "Invalid UUID format. Use full UUID or short format (first 8 characters) from 'one-time list'"
    )
}

/// Executes export command
pub async fn execute_export(format: &str) -> Result<()> {
    let data = load_data().await?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&data)?;
            println!("{}", json);
        }
        "csv" => {
            // Simple CSV export of all transactions
            println!("Type,Description,Amount,Date/Day,Active");

            for txn in &data.recurring {
                println!(
                    "recurring,\"{}\",{},{},{}",
                    txn.description, txn.amount, txn.day_of_month, txn.active
                );
            }

            for txn in &data.one_time {
                println!(
                    "one-time,\"{}\",{},{}",
                    txn.description, txn.amount, txn.date
                );
            }
        }
        _ => {
            anyhow::bail!("Unsupported format: {}. Use 'json' or 'csv'", format);
        }
    }

    Ok(())
}
