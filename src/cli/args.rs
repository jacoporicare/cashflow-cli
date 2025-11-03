use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "cashflow")]
#[command(version = "0.1.0")]
#[command(about = "Cashflow planning for recurring payments", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show cashflow projection for next N days
    Plan {
        /// Number of days to project (default: 30)
        #[arg(short, long, default_value_t = 30)]
        days: i64,
    },

    /// Manage account balance
    Balance {
        #[command(subcommand)]
        action: BalanceAction,
    },

    /// Manage recurring transactions
    #[command(alias = "rec")]
    Recurring {
        #[command(subcommand)]
        action: RecurringAction,
    },

    /// Manage one-time transactions
    #[command(alias = "one")]
    OneTime {
        #[command(subcommand)]
        action: OneTimeAction,
    },

    /// Export data
    Export {
        /// Format: csv, json
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Manage configuration
    #[command(alias = "conf")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show,

    /// Set data directory path
    SetDataDir {
        /// Path to data directory
        path: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum BalanceAction {
    /// Set current balance
    Set {
        /// Balance amount
        amount: String,

        /// Date (format: DD.MM.YYYY or YYYY-MM-DD), defaults to today
        #[arg(long)]
        date: Option<String>,
    },

    /// Show current balance
    Show,
}

#[derive(Subcommand, Debug)]
pub enum RecurringAction {
    /// Add a new recurring transaction
    Add {
        /// Description
        #[arg(short, long)]
        description: String,

        /// Amount (positive for income, negative for expense)
        #[arg(short, long, allow_negative_numbers = true)]
        amount: String,

        /// Day of month (1-31)
        #[arg(long)]
        day: u8,
    },

    /// List all recurring transactions
    List,

    /// Edit a recurring transaction
    Edit {
        /// Transaction ID
        id: String,

        /// New amount
        #[arg(short, long, allow_negative_numbers = true)]
        amount: Option<String>,

        /// New day of month
        #[arg(long)]
        day: Option<u8>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Disable a recurring transaction
    Disable {
        /// Transaction ID
        id: String,
    },

    /// Enable a recurring transaction
    Enable {
        /// Transaction ID
        id: String,
    },

    /// Delete a recurring transaction permanently
    #[command(alias = "del")]
    Delete {
        /// Transaction ID
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum OneTimeAction {
    /// Add a new one-time transaction
    Add {
        /// Description
        #[arg(short, long)]
        description: String,

        /// Amount (positive for income, negative for expense)
        #[arg(short, long, allow_negative_numbers = true)]
        amount: String,

        /// Date (format: DD.MM.YYYY or YYYY-MM-DD)
        #[arg(long)]
        date: String,
    },

    /// List one-time transactions
    List {
        /// Show only upcoming transactions
        #[arg(long)]
        upcoming: bool,
    },

    /// Edit a one-time transaction
    Edit {
        /// Transaction ID
        id: String,

        /// New amount
        #[arg(short, long, allow_negative_numbers = true)]
        amount: Option<String>,

        /// New date (format: DD.MM.YYYY or YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Delete a one-time transaction permanently
    #[command(alias = "del")]
    Delete {
        /// Transaction ID
        id: String,
    },
}
