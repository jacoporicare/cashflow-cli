use crate::Result;
use crate::cli::format_amount;
use crate::commands::balance::parse_amount;
use crate::models::RecurringTransaction;
use crate::storage::{load_data, save_data};
use anyhow::Context;
use comfy_table::{Attribute, Cell, Color, Table, presets::UTF8_FULL};
use uuid::Uuid;

/// Executes recurring add command
pub async fn execute_recurring_add(description: &str, amount_str: &str, day: u8) -> Result<()> {
    // Validate day
    if !(1..=31).contains(&day) {
        anyhow::bail!("Day must be between 1 and 31");
    }

    // Parse amount
    let amount = parse_amount(amount_str)?;

    // Load data
    let mut data = load_data().await?;

    // Create recurring transaction
    let transaction = RecurringTransaction::new(description.to_string(), amount, day);

    data.recurring.push(transaction.clone());

    // Save data
    save_data(&data).await?;

    println!("Added recurring transaction:");
    println!("  Description: {}", transaction.description);
    println!("  Amount: {}", format_amount(transaction.amount));
    println!("  Day of month: {}", transaction.day_of_month);
    println!("  ID: {}", transaction.id);

    Ok(())
}

/// Executes recurring list command
pub async fn execute_recurring_list() -> Result<()> {
    let data = load_data().await?;

    if data.recurring.is_empty() {
        println!("No recurring transactions found.");
        println!("Add one with:");
        println!("  cashflow recurring add -d <description> -a <amount> --day <day>");
        return Ok(());
    }

    // Sort by day of month
    let mut transactions = data.recurring.clone();
    transactions.sort_by_key(|t| t.day_of_month);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold),
        Cell::new("Description").add_attribute(Attribute::Bold),
        Cell::new("Amount").add_attribute(Attribute::Bold),
        Cell::new("Day").add_attribute(Attribute::Bold),
        Cell::new("Active").add_attribute(Attribute::Bold),
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
            Cell::new(txn.day_of_month.to_string()),
            Cell::new(if txn.active { "✓" } else { "✗" }).fg(if txn.active {
                Color::Green
            } else {
                Color::Red
            }),
        ]);
    }

    println!("{table}");
    println!();
    println!("Total: {} recurring transactions", data.recurring.len());

    Ok(())
}

/// Executes recurring disable command
pub async fn execute_recurring_disable(id_str: &str) -> Result<()> {
    let mut data = load_data().await?;
    let id = parse_uuid(id_str, &data.recurring)?;

    let transaction = data
        .recurring
        .iter_mut()
        .find(|t| t.id == id)
        .context("Recurring transaction not found")?;

    transaction.active = false;
    let description = transaction.description.clone();

    save_data(&data).await?;

    println!("Disabled recurring transaction: {}", description);

    Ok(())
}

/// Executes recurring enable command
pub async fn execute_recurring_enable(id_str: &str) -> Result<()> {
    let mut data = load_data().await?;
    let id = parse_uuid(id_str, &data.recurring)?;

    let transaction = data
        .recurring
        .iter_mut()
        .find(|t| t.id == id)
        .context("Recurring transaction not found")?;

    transaction.active = true;
    let description = transaction.description.clone();

    save_data(&data).await?;

    println!("Enabled recurring transaction: {}", description);

    Ok(())
}

/// Executes recurring delete command
pub async fn execute_recurring_delete(id_str: &str) -> Result<()> {
    let mut data = load_data().await?;
    let id = parse_uuid(id_str, &data.recurring)?;

    let index = data
        .recurring
        .iter()
        .position(|t| t.id == id)
        .context("Recurring transaction not found")?;

    let removed = data.recurring.remove(index);
    save_data(&data).await?;

    println!("Deleted recurring transaction: {}", removed.description);

    Ok(())
}

/// Executes recurring edit command
pub async fn execute_recurring_edit(
    id_str: &str,
    amount: Option<&str>,
    day: Option<u8>,
    description: Option<&str>,
) -> Result<()> {
    let mut data = load_data().await?;
    let id = parse_uuid(id_str, &data.recurring)?;

    let transaction = data
        .recurring
        .iter_mut()
        .find(|t| t.id == id)
        .context("Recurring transaction not found")?;

    if let Some(amount_str) = amount {
        transaction.amount = parse_amount(amount_str)?;
        println!("Updated amount: {}", format_amount(transaction.amount));
    }

    if let Some(d) = day {
        if !(1..=31).contains(&d) {
            anyhow::bail!("Day must be between 1 and 31");
        }
        transaction.day_of_month = d;
        println!("Updated day of month: {}", d);
    }

    if let Some(desc) = description {
        transaction.description = desc.to_string();
        println!("Updated description: {}", desc);
    }

    save_data(&data).await?;
    println!("Recurring transaction updated successfully.");

    Ok(())
}

/// Parses UUID from string, supports both full and short (8 char) format
fn parse_uuid(s: &str, transactions: &[RecurringTransaction]) -> Result<Uuid> {
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
                    "No recurring transaction found with ID starting with '{}'. Use 'recurring list' to see available IDs.",
                    s
                )
            }
            1 => return Ok(matching[0].id),
            _ => {
                anyhow::bail!(
                    "Multiple transactions match '{}'. Use full UUID to be more specific. Use 'recurring list' to see all IDs.",
                    s
                )
            }
        }
    }

    anyhow::bail!(
        "Invalid UUID format. Use full UUID or short format (first 8 characters) from 'recurring list'"
    )
}
