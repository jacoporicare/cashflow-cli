# Cashflow CLI - Personal Finance Manager

Cashflow planning tool for managing recurring payments on a dedicated bank account.

## Overview

Cashflow CLI helps you track and project your cashflow for accounts with regular scheduled payments (direct debits, standing orders, subscriptions). Know exactly how much money you need on any given day.

**Use Case**: Separate account for recurring payments (mortgage, Netflix, utilities) vs. daily expenses account.

## Features

- 📊 **30/60-day cashflow projection** with running balance
- 🔄 **Recurring transactions** (monthly subscriptions, bills)
- 💚 **One-time transactions** (transfers, one-off expenses)
- 💰 **Balance management** - set balance on any date
- 📈 **Beautiful terminal UI** with color-coded amounts
- 💾 **Human-readable storage** - RON format, git-friendly
- 📤 **Export** to JSON/CSV

## Installation

```bash
cargo build --release
sudo cp target/release/cashflow /usr/local/bin/
```

## Data Directory & Safety

**Important: Development vs Production Data**

To prevent accidentally overwriting your production data during development, the tool automatically uses different data directories:

- **Development** (`cargo run`): `./dev-data/` (local to repo, git-ignored)
- **Production** (release binary): `~/.cashflow/` or custom directory from `~/.cashflowrc`
- **Override**: Use `CASHFLOW_DATA_DIR=/path/to/data` environment variable

```bash
# Development - uses ./dev-data/ (your .cashflowrc is ignored)
cargo run -- plan

# Production - uses ~/.cashflow/ or your custom directory
cashflow plan

# Override for this session
CASHFLOW_DATA_DIR=/path/to/data cargo run -- plan

# Check which directory is being used
cargo run -- config show
```

**Your production data is always safe during development!** 🛡️

## Quick Start

### 1. Set your current balance

```bash
cashflow balance set 5000 --date="01.01.2025"
```

### 2. Add recurring transactions

```bash
# Expenses (use negative amounts with equals sign)
cashflow recurring add --description="Netflix" --amount=-15 --day=14
cashflow recurring add --description="Mortgage" --amount=-1200 --day=20
# Or use shortcut: cashflow rec add ...

# Income
cashflow recurring add --description="Salary" --amount=5000 --day=1
```

### 3. View your cashflow projection

```bash
cashflow plan        # Next 30 days
cashflow plan --days 60  # Next 60 days
```

## Commands

### Plan

Show cashflow projection:

```bash
cashflow plan            # 30 days (default)
cashflow plan --days 60  # Custom period
```

### Balance

Manage account balance:

```bash
cashflow balance set 5000                    # Set balance for today
cashflow balance set 5000 --date="01.01.2025"  # Set for specific date
cashflow balance show                         # Show current balance
```

### Recurring Transactions

Manage monthly recurring payments:

```bash
# Add (can use shortcut: cashflow rec add)
cashflow recurring add -d "Description" --amount=-50 --day=14 -c "Category"

# List all (can use shortcut: cashflow rec list)
cashflow recurring list

# Edit
cashflow recurring edit <id> --amount=-500
cashflow recurring edit <id> --day=15

# Disable/Enable (keep but don't project)
cashflow recurring disable <id>
cashflow recurring enable <id>

# Delete permanently
cashflow recurring delete <id>
```

**Note**: For negative amounts, use the format `--amount=-50` (equals sign, no space).  
**Shortcut**: Use `rec` instead of `recurring` (e.g., `cashflow rec add`).

### One-Time Transactions

Add, list, edit, and delete one-time transactions:

```bash
# Add (can use shortcut: cashflow one add)
cashflow one-time add -d "Transfer from savings" -a 2000 --date="15.01.2025"
cashflow one-time add -d "Car repair" --amount=-500 --date="20.01.2025" -c "Maintenance"

# List all (can use shortcut: cashflow one list)
cashflow one-time list

# List upcoming only
cashflow one-time list --upcoming

# Edit
cashflow one-time edit <id> --amount=-100
cashflow one-time edit <id> --date="20.01.2025"

# Delete
cashflow one-time delete <id>
```

**Shortcut**: Use `one` instead of `one-time` (e.g., `cashflow one add`).

### Export

Export data:

```bash
cashflow export --format json > backup.json
cashflow export --format csv > transactions.csv
```

### Configuration

Manage data directory location:

```bash
# Show current configuration
cashflow config show

# Change data directory
cashflow config set-data-dir ~/Documents/cashflow-data
cashflow config set-data-dir /path/to/custom/location
```

Configuration is stored in `~/.cashflowrc` (TOML format).

## Data Storage

Data is stored in `~/.cashflow/data.ron` (or custom location set via `cashflow config set-data-dir`) in human-readable RON format:

**Configuration**: `~/.cashflowrc` stores your data directory path (default: `~/.cashflow`)

```ron
(
    recurring: [
        (
            id: "550e8400-e29b-41d4-a716-446655440001",
            description: "Netflix",
            amount: "-15",
            day_of_month: 14,
            active: true,
            created_at: "2025-01-01T12:00:00Z",
        ),
    ],
    one_time: [...],
    balance_snapshots: [...],
)
```

**Benefits:**
- ✅ Git-friendly (line-by-line diffs)
- ✅ Easy to edit manually
- ✅ Simple backup: `cp ~/.cashflow/data.ron ~/backup/`
- ✅ Comments supported with `//`

## Example Output

```
┌──────────────┬──────────────────────┬───────────┬──────────────┐
│ Datum        │ Popis                │ Částka    │ Zůstatek     │
╞══════════════╪══════════════════════╪═══════════╪══════════════╡
│ 13.01.2025   │ Současný zůstatek    │           │   10 000 Kč  │
│ 14.01.2025   │ Netflix              │    -15 Kč │    9 985 Kč  │
│ 15.01.2025   │ Salary               │  5 000 Kč │   14 985 Kč  │
│ 15.01.2025   │ Phone bill           │    -50 Kč │   14 935 Kč  │
│ 20.01.2025   │ Utilities            │   -300 Kč │   14 635 Kč  │
│ 20.01.2025   │ Pension              │   -200 Kč │   14 435 Kč  │
│ 20.01.2025   │ Investment           │   -500 Kč │   13 935 Kč  │
│ 20.01.2025   │ Mortgage             │ -1 200 Kč │   12 735 Kč  │
│ 27.01.2025   │ Spotify              │    -10 Kč │   12 725 Kč  │
│ 28.01.2025   │ Transfer             │  2 000 Kč │   14 725 Kč  │
└──────────────┴──────────────────────┴───────────┴──────────────┘

Celkem za období: 4 725 Kč
Nejnižší zůstatek: 9 985 Kč (14.01.2025)

💚 = jednorázová transakce (one-time)
⚠️  = zůstatek pod 10 000 Kč
```

## Technical Details

- **Language**: Rust 2024 edition
- **Storage**: RON (Rusty Object Notation)
- **CLI**: clap v4
- **Decimal**: rust_decimal for precise money calculations
- **Date/Time**: chrono
- **Tables**: comfy-table with UTF-8 borders

## Development

```bash
# Build
cargo build

# Run
cargo run -- plan

# Run tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Advantages over Spreadsheets

✅ No need to manually delete one-time transactions  
✅ Set balance on ANY date, not just specific days  
✅ Automatic projection of recurring payments  
✅ Clear separation of recurring vs one-time  
✅ Fast queries, no formula recalculation  
✅ Git-friendly data format  

## Future Enhancements

- [ ] Import from bank statements (CSV)
- [ ] Reconciliation mode (actual vs planned)
- [ ] Custom recurring patterns (bi-weekly, quarterly)
- [ ] Balance alerts and notifications
- [ ] Interactive TUI mode
- [ ] Charts and visualizations

## License

MIT

## Author

Built with ❤️ in Rust

