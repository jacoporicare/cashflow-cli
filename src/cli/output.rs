use crate::models::TransactionView;
use crate::projection::ProjectedCashflow;
use chrono::NaiveDate;
use colored::*;
use comfy_table::{Attribute, Cell, CellAlignment, Color, Table, presets::UTF8_FULL};
use rust_decimal::Decimal;

/// Display options for the cashflow plan table
pub struct PlanDisplayOptions {
    pub warning_threshold: Decimal,
    pub show_past: bool,
}

/// Formats a decimal amount as Czech currency (e.g., "22 158 Kč")
pub fn format_amount(amount: Decimal) -> String {
    let abs_amount = amount.abs();
    let amount_str = abs_amount.to_string();

    // Split into integer and decimal parts
    let parts: Vec<&str> = amount_str.split('.').collect();
    let integer_part = parts[0];

    // Add thousand separators (space)
    let formatted = add_thousand_separators(integer_part);

    // Add sign and currency
    if amount.is_sign_negative() {
        format!("-{} Kč", formatted)
    } else {
        format!("{} Kč", formatted)
    }
}

/// Adds thousand separators to a number string
fn add_thousand_separators(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, c) in chars.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(' ');
        }
        result.push(*c);
    }

    result.chars().rev().collect()
}

/// Formats a date in Czech format (DD.MM.YYYY)
pub fn format_date(date: NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}

/// Adds a transaction row to the table and updates minimum balance tracking
fn add_transaction_row(
    table: &mut Table,
    txn: &TransactionView,
    warning_threshold: Decimal,
    min_balance: &mut Decimal,
    min_balance_date: &mut NaiveDate,
    is_past: bool,
) {
    let amount_cell = Cell::new(format_amount(txn.amount))
        .set_alignment(CellAlignment::Right)
        .fg(if txn.amount.is_sign_negative() {
            Color::Red
        } else {
            Color::Green
        });

    let balance_cell = Cell::new(format_amount(txn.balance_after))
        .set_alignment(CellAlignment::Right)
        .fg(if txn.balance_after.is_sign_negative() {
            Color::Red
        } else if txn.balance_after < warning_threshold {
            Color::Yellow
        } else {
            Color::Cyan
        });

    let mut description = txn.description.clone();
    if txn.is_one_time {
        description.push_str(" 💚");
    }

    table.add_row(vec![
        Cell::new(format_date(txn.date)).fg(if is_past {
            Color::DarkGrey
        } else {
            Color::White
        }),
        Cell::new(description).fg(if is_past {
            Color::DarkGrey
        } else {
            Color::White
        }),
        amount_cell,
        balance_cell,
    ]);

    // Track minimum balance
    if txn.balance_after < *min_balance {
        *min_balance = txn.balance_after;
        *min_balance_date = txn.date;
    }
}

/// Prints the cashflow projection table
pub fn print_plan_table(projection: &ProjectedCashflow, options: &PlanDisplayOptions) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec![
            Cell::new("Datum").add_attribute(Attribute::Bold),
            Cell::new("Popis").add_attribute(Attribute::Bold),
            Cell::new("Částka").add_attribute(Attribute::Bold),
            Cell::new("Zůstatek").add_attribute(Attribute::Bold),
        ])
        .add_row(vec![
            Cell::new(format_date(projection.balance.date)).fg(Color::DarkGrey),
            Cell::new("Nastavený zůstatek")
                .fg(Color::DarkGrey)
                .add_attribute(Attribute::Bold),
            Cell::new(""),
            Cell::new(format_amount(projection.balance.balance))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Cyan),
        ]);

    // Track minimum balance across all transactions
    let mut min_balance = projection.starting_balance;
    let mut min_balance_date = projection.start_date;

    if options.show_past {
        for txn in &projection.past_txns {
            add_transaction_row(
                &mut table,
                txn,
                options.warning_threshold,
                &mut min_balance,
                &mut min_balance_date,
                true,
            );
        }
    }

    // Then, add the current balance row
    table.add_row(vec![
        Cell::new(format_date(projection.start_date)),
        Cell::new("Současný zůstatek").add_attribute(Attribute::Bold),
        Cell::new(""),
        Cell::new(format_amount(projection.starting_balance))
            .set_alignment(CellAlignment::Right)
            .fg(Color::Cyan),
    ]);

    // Finally, add projected (future) transactions
    for txn in &projection.future_txns {
        add_transaction_row(
            &mut table,
            txn,
            options.warning_threshold,
            &mut min_balance,
            &mut min_balance_date,
            false,
        );
    }

    println!("{table}");
    println!();

    // Summary
    if let Some(last) = projection.future_txns.last() {
        let total_change = last.balance_after - projection.starting_balance;
        let total_str = format_amount(total_change);

        if total_change.is_sign_negative() {
            println!("Celkem za období: {}", total_str.red());
        } else {
            println!("Celkem za období: {}", total_str.green());
        }
    }

    if min_balance < options.warning_threshold {
        println!(
            "Nejnižší zůstatek: {} ({})",
            format_amount(min_balance).yellow(),
            format_date(min_balance_date)
        );
    } else {
        println!(
            "Nejnižší zůstatek: {} ({})",
            format_amount(min_balance),
            format_date(min_balance_date)
        );
    }

    println!();
    println!("💚 = jednorázová transakce (one-time)");
    println!(
        "⚠️  = zůstatek pod {}",
        format_amount(options.warning_threshold)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_thousand_separators() {
        assert_eq!(add_thousand_separators("1000"), "1 000");
        assert_eq!(add_thousand_separators("22158"), "22 158");
        assert_eq!(add_thousand_separators("1000000"), "1 000 000");
        assert_eq!(add_thousand_separators("123"), "123");
    }

    #[test]
    fn test_format_amount() {
        use std::str::FromStr;

        assert_eq!(
            format_amount(Decimal::from_str("22158").unwrap()),
            "22 158 Kč"
        );
        assert_eq!(format_amount(Decimal::from_str("-478").unwrap()), "-478 Kč");
        assert_eq!(
            format_amount(Decimal::from_str("1000000").unwrap()),
            "1 000 000 Kč"
        );
    }
}
