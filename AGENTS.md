You are an expert in Rust, async programming, and concurrent systems.

**Rust Edition**: Use **Rust 2024 edition** exclusively. Write modern, idiomatic Rust code following latest best practices.

Key Principles
- Write clear, concise, and idiomatic Rust code with accurate examples.
- Use Rust 2024 edition features and modern idioms.
- Follow best practices from The Rust Book, Rust API Guidelines, and Clippy recommendations.
- Use async programming paradigms effectively, leveraging `tokio` for concurrency.
- Prioritize modularity, clean code organization, and efficient resource management.
- Use expressive variable names that convey intent (e.g., `is_ready`, `has_data`).
- Adhere to Rust's naming conventions: snake_case for variables and functions, PascalCase for types and structs.
- Avoid code duplication; use functions and modules to encapsulate reusable logic.
- Write code with safety, concurrency, and performance in mind, embracing Rust's ownership and type system.
- Prefer iterator methods over loops where idiomatic.
- Use `?` operator for error propagation instead of match/unwrap.
- Leverage type inference where clear, explicit types where necessary for documentation.

Function Parameters: String vs &str
- **Prefer `&str` over `String` for function parameters** when you only need to read the string.
- This is more flexible for callers: they can pass `&my_string`, `"literal"`, or `&my_string[0..5]` without cloning.
- Only convert to `String` (using `.to_string()`) when you actually need to store or own the data.
- For `Option<String>` parameters, accept `Option<&str>` and use `.as_deref()` when calling from code that borrows.
- For `&str`, no `&` prefix needed when passing to functions that already take references (e.g., `parse_amount(amount_str)` not `parse_amount(&amount_str)`).
- **Rule of thumb**: If you only **read** the string → use `&str`; if you need to **store/modify** → convert to `String` at that point.
- **Performance**: Using `&str` avoids unnecessary heap allocations and memory copies.

Example:
```rust
// ✅ GOOD: Accept &str, convert to String only when storing
pub async fn execute_add(description: &str, amount_str: &str) -> Result<()> {
    let amount = parse_amount(amount_str)?;  // No & needed, amount_str is already &str
    let transaction = Transaction::new(description.to_string(), amount);  // Convert to String only when storing
    // ...
}

// ❌ BAD: Accepting String forces callers to clone
pub async fn execute_add(description: String, amount_str: String) -> Result<()> {
    let amount = parse_amount(&amount_str)?;  // Need & because parameter is String
    let transaction = Transaction::new(description, amount);
    // ...
}

// Calling from match on &cli.command:
match &cli.command {
    Command::Add { description, amount } => {
        execute_add(&description, &amount).await?;  // Just borrow, no cloning
    }
}
```

Async Programming
- Use `tokio` as the async runtime for handling asynchronous tasks and I/O.
- Implement async functions using `async fn` syntax.
- Leverage `tokio::spawn` for task spawning and concurrency.
- Use `tokio::select!` for managing multiple async tasks and cancellations.
- Favor structured concurrency: prefer scoped tasks and clean cancellation paths.
- Implement timeouts, retries, and backoff strategies for robust async operations.

Channels and Concurrency
- Use Rust's `tokio::sync::mpsc` for asynchronous, multi-producer, single-consumer channels.
- Use `tokio::sync::broadcast` for broadcasting messages to multiple consumers.
- Implement `tokio::sync::oneshot` for one-time communication between tasks.
- Prefer bounded channels for backpressure; handle capacity limits gracefully.
- Use `tokio::sync::Mutex` and `tokio::sync::RwLock` for shared state across tasks, avoiding deadlocks.

Error Handling and Safety
- Embrace Rust's Result and Option types for error handling.
- Use `?` operator to propagate errors in async functions.
- Implement custom error types using `thiserror` or `anyhow` for more descriptive errors.
- Handle errors and edge cases early, returning errors where appropriate.
- Use `.await` responsibly, ensuring safe points for context switching.

Testing
- Write unit tests with `tokio::test` for async tests.
- Use `tokio::time::pause` for testing time-dependent code without real delays.
- Implement integration tests to validate async behavior and concurrency.
- Use mocks and fakes for external dependencies in tests.

Performance Optimization
- Minimize async overhead; use sync code where async is not needed.
- Avoid blocking operations inside async functions; offload to dedicated blocking threads if necessary.
- Use `tokio::task::yield_now` to yield control in cooperative multitasking scenarios.
- Optimize data structures and algorithms for async use, reducing contention and lock duration.
- Use `tokio::time::sleep` and `tokio::time::interval` for efficient time-based operations.

Key Conventions
1. **Rust 2024 Edition**: Always use `edition = "2024"` in Cargo.toml.
2. Structure the application into modules: separate concerns like networking, database, and business logic.
3. Use modern module organization: `module.rs` + `module/submodule.rs` instead of `module/mod.rs` (old style).
4. Follow Rust API Guidelines and idiomatic patterns.
5. Use environment variables for configuration management (e.g., `dotenv` crate).
6. Ensure code is well-documented with inline comments and Rustdoc.
7. Run `cargo clippy` regularly and address warnings.
8. Use `cargo fmt` for consistent formatting.

Async Ecosystem
- Use `tokio` for async runtime and task management.
- Leverage `hyper` or `reqwest` for async HTTP requests.
- Use `serde` for serialization/deserialization.
- Use `sqlx` or `tokio-postgres` for async database interactions.
- Utilize `tonic` for gRPC with async support.

Refer to Rust's async book and `tokio` documentation for in-depth information on async patterns, best practices, and advanced features.

---

## Project: Cashflow CLI - Personal Finance Manager

### Overview
Cashflow is a command-line interface tool for **cashflow planning and tracking recurring payments** on a dedicated account (e.g., Savings Account). The primary goal is to know exactly how much money needs to be in the account on any given day to cover all scheduled payments.

**Use Case**: User has two bank accounts:
- **Savings Account** - for regular scheduled payments (direct debits, standing orders, mortgage, subscriptions like Netflix)
- **Checking Account** - for daily expenses (shopping, restaurants)

This tool manages the Savings Account to ensure sufficient balance for all recurring obligations.

### Core Features
1. **Recurring Transactions (Pravidelné platby)**
   - Define monthly recurring payments (e.g., Netflix on 14th, mortgage on 20th)
   - Automatically project recurring transactions into the future
   - Edit recurring transaction templates (change amount, day, description)
   - Distinguish between one-time and recurring transactions
   - Mark transactions as recurring or one-time only

2. **One-Time Transactions (Jednorázové)**
   - Add one-time income (e.g., "Transfer from checking account")
   - Add one-time expenses
   - Automatically removed after their date passes (configurable retention)
   - Clear distinction from recurring transactions

3. **Cashflow Planning & Projection**
   - **`plan` command**: Show projected balance for next 30 days from today
   - Display date, transaction description, amount, and running balance
   - Set current account balance for any date (not just 13th of month)
   - Automatic calculation of future balance based on scheduled transactions
   - Warning when balance drops below zero or threshold

4. **Balance Management**
   - Set current balance on any date
   - Track running balance (zůstatek) day by day
   - Calculate required balance for upcoming obligations
   - Show minimum balance needed to cover all scheduled payments

5. **User Experience**
   - Fast, responsive CLI with minimal latency
   - Intuitive command structure (plan, add, recurring, balance, list)
   - Support for Czech language in output and descriptions
   - Color-coded output: income (green), expenses (red), warnings (yellow)
   - Table format matching Numbers spreadsheet layout

### Data Model
```rust
// Top-level RON data structure (~/.cashflow/data.ron)
#[derive(Debug, Serialize, Deserialize)]
struct CashflowData {
    recurring: Vec<RecurringTransaction>,
    one_time: Vec<OneTimeTransaction>,
    balance_snapshots: Vec<BalanceSnapshot>,
}

// Recurring transaction template
#[derive(Debug, Serialize, Deserialize)]
struct RecurringTransaction {
    id: Uuid,
    description: String,
    amount: Decimal,           // Positive for income, negative for expenses
    day_of_month: u8,          // 1-31, day when payment occurs
    active: bool,              // Can be disabled without deleting
    created_at: DateTime<Utc>,
}

// One-time transaction
#[derive(Debug, Serialize, Deserialize)]
struct OneTimeTransaction {
    id: Uuid,
    description: String,
    amount: Decimal,
    date: NaiveDate,
    created_at: DateTime<Utc>,
}

// Balance snapshot - current balance at a specific date
#[derive(Debug, Serialize, Deserialize)]
struct BalanceSnapshot {
    id: Uuid,
    date: NaiveDate,
    balance: Decimal,
    created_at: DateTime<Utc>,
}

// Projected transaction (runtime only, not persisted)
#[derive(Debug)]
struct ProjectedTransaction {
    date: NaiveDate,
    day_of_month: u8,
    description: String,
    amount: Decimal,
    is_one_time: bool,         // True if from one_time, false if recurring
    balance_after: Decimal,    // Calculated during projection
}
```

### Architecture Principles
1. **Storage - Hybrid Approach (RON + SQLite)**
   - **Primary storage**: RON file (`~/.cashflow/data.ron`) - human-readable, git-friendly, source of truth
   - **Cache layer**: SQLite (`~/.cashflow/cache.db`) - fast queries, automatic regeneration
   - RON file structure: `recurring`, `one_time`, `balance_snapshots` collections
   - SQLite automatically rebuilt from RON when file changes detected
   - Three main tables: `recurring_transactions`, `transactions`, `balance_snapshots`
   - Manual editing: users can edit data.ron directly in any text editor
   - Backup-friendly: simple text file, easy to version control
   - Cache invalidation: detect RON file mtime changes, rebuild cache if needed

2. **Cache Invalidation Strategy**
   - On every command, check RON file mtime (modification time)
   - Compare with last cached mtime stored in SQLite metadata table
   - If RON changed: deserialize entire file, rebuild SQLite cache
   - If RON unchanged: use SQLite cache directly (fast path)
   - Write operations: update RON file atomically, then invalidate cache
   - Atomic writes: write to `.data.ron.tmp`, then rename to `data.ron`

3. **Cashflow Projection Logic**
   - Start from most recent balance snapshot
   - Generate projected transactions from recurring templates for next N days
   - Merge with one-time transactions
   - Sort by date, calculate running balance for each day
   - Handle month boundaries (e.g., day 31 in months with 30 days)
   - Efficiently cache projections for performance

4. **CLI Framework**
   - Use `clap` for command-line argument parsing
   - Main subcommands: `plan`, `balance`, `recurring` (alias: `rec`), `one-time` (alias: `one`), `export`, `config`
   - `recurring` subcommands: `add`, `list`, `edit`, `disable`, `enable`, `delete`
   - `one-time` subcommands: `add`, `list`, `edit`, `delete`
   - Support both interactive and script-friendly modes
   - Pipe-friendly output for scripting

5. **Display & Formatting**
   - Use `comfy-table` or `tabled` for formatted table output
   - Use `colored` for terminal color support
   - Format currency with proper thousands separators (e.g., "5 000 Kč")
   - Align numbers right, text left in tables
   - Columns: Datum (date), Popis (description), Částka (amount), Zůstatek (balance)
   - Show description clearly

6. **Data Validation**
   - Validate dates and amounts before insertion
   - Validate day_of_month (1-31)
   - Handle edge cases (leap years, month boundaries, day 31 in 30-day months)
   - Validate UTF-8 for Czech characters
   - Prevent invalid balance snapshots

7. **Configuration**
   - Store user preferences in ~/.cashflow/config.toml
   - Support configuration for default currency, date format, locale
   - Allow customization of categories and aliases
   - Configurable warning threshold for low balance
   - Configurable retention period for past one-time transactions

### Example RON Data File

`~/.cashflow/data.ron`:
```ron
(
    recurring: [
        (
            id: "550e8400-e29b-41d4-a716-446655440001",
            description: "Netflix",
            amount: -15,
            day_of_month: 14,
            active: true,
            created_at: (2025, 1, 1, 12, 0, 0, 0),
        ),
        (
            id: "550e8400-e29b-41d4-a716-446655440002",
            description: "Salary",
            amount: 5000,
            day_of_month: 1,
            active: true,
            created_at: (2025, 1, 1, 12, 0, 0, 0),
        ),
        (
            id: "550e8400-e29b-41d4-a716-446655440003",
            description: "Mortgage",
            amount: -1200,
            day_of_month: 20,
            active: true,
            created_at: (2025, 1, 1, 12, 0, 0, 0),
        ),
        (
            id: "550e8400-e29b-41d4-a716-446655440004",
            description: "Spotify",
            amount: -10,
            day_of_month: 27,
            active: true,
            created_at: (2025, 1, 1, 12, 0, 0, 0),
        ),
    ],
    one_time: [
        (
            id: "650e8400-e29b-41d4-a716-446655440001",
            description: "Transfer from savings",
            amount: 2000,
            date: (2025, 1, 25),
            created_at: (2025, 1, 10, 15, 30, 0, 0),
        ),
    ],
    balance_snapshots: [
        (
            id: "750e8400-e29b-41d4-a716-446655440001",
            date: (2025, 1, 1),
            balance: 5000,
            created_at: (2025, 1, 1, 10, 0, 0, 0),
        ),
    ],
)
```

**Benefits:**
- ✅ Easy to read and manually edit
- ✅ Git-friendly: line-by-line diffs
- ✅ Comments supported with `//`
- ✅ Simple backup: `cp ~/.cashflow/data.ron ~/backup/`

### Example Output - `cashflow plan`

The main command `cashflow plan` should output a table similar to the Numbers spreadsheet:

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
│ 30.01.2025   │ Cloud storage        │    -10 Kč │   14 715 Kč  │
│ 30.01.2025   │ Hosting              │    -20 Kč │   14 695 Kč  │
│ 01.02.2025   │ TV license           │    -30 Kč │   14 665 Kč  │
│ 01.02.2025   │ Government fees      │   -400 Kč │   14 265 Kč  │
│ 01.02.2025   │ Rent                 │ -3 000 Kč │   11 265 Kč  │
└──────────────┴──────────────────────┴───────────┴──────────────┘

Celkem za období: 1 265 Kč
Nejnižší zůstatek: 9 985 Kč (14.01.2025)

💚 = jednorázová transakce (one-time)
⚠️  = zůstatek pod 10 000 Kč
```

**Color coding:**
- Income (positive amounts): GREEN
- Expenses (negative amounts): RED
- Low balance warning: YELLOW
- One-time transactions: Different symbol/marker

### CLI Command Examples
```bash
# Show 30-day cashflow plan from today
cashflow plan
cashflow plan --days 60  # Show 60 days ahead

# Set current balance (any date)
cashflow balance set 5000
cashflow balance set 5000 --date "01.01.2025"

# Show current balance
cashflow balance

# Add recurring transaction (will appear every month)
cashflow recurring add -d "Netflix" -a -15 --day 14
cashflow recurring add -d "Mortgage" -a -1200 --day 20
cashflow recurring add -d "Spotify" -a -10 --day 27
# Or use shortcut: cashflow rec add ...

# List all recurring transactions
cashflow recurring list
# Or: cashflow rec list

# Edit recurring transaction
cashflow recurring edit <id> -a -500
cashflow recurring edit <id> --day 15

# Disable/enable recurring transaction (without deleting)
cashflow recurring disable <id>
cashflow recurring enable <id>

# Delete recurring transaction permanently
cashflow recurring delete <id>

# Add one-time transaction
cashflow one-time add -d "Transfer from savings" -a 2000 --date "25.01.2025"
cashflow one-time add -d "One-time repair" -a -500 --date "15.02.2025"
# Or use shortcut: cashflow one add ...

# List one-time transactions
cashflow one-time list
cashflow one-time list --upcoming  # Only future transactions
# Or: cashflow one list

# Edit one-time transaction
cashflow one-time edit <id> -a -100
cashflow one-time edit <id> --date "20.01.2025"

# Delete one-time transaction
cashflow one-time delete <id>

# Export data
cashflow export --format csv > transactions.csv
cashflow export --format json > backup.json
```

### Technical Stack
- **CLI**: `clap` v4+ with derive macros
- **Storage (Primary)**: `ron` v0.8 for human-readable data format
- **Storage (Cache)**: `sqlx` v0.7 with SQLite for fast async queries
- **Date/Time**: `chrono` for date handling
- **Decimal**: `rust_decimal` for precise monetary calculations
- **Tables**: `comfy-table` for formatted output
- **Colors**: `colored` or `owo-colors` for terminal styling
- **Serialization**: `serde` v1.0 with derive macros for RON/JSON
- **Config**: `toml` for configuration files (~/.cashflow/config.toml)
- **Error Handling**: `anyhow` for application errors, `thiserror` for library errors
- **UUID**: `uuid` v1.0 with v4 and serde features
- **Async Runtime**: `tokio` with full features

### Module Organization
Use modern Rust module structure (Rust 2018+ edition):

```
src/
  lib.rs              # Library root, declares all modules
  main.rs             # Binary entry point
  models.rs           # Models module root
  models/
    data.rs           # Data structures
  storage.rs          # Storage module root
  storage/
    ron_storage.rs    # RON file operations
  projection.rs       # Projection module root
  projection/
    cashflow.rs       # Cashflow projection logic
  cli.rs              # CLI module root
  cli/
    args.rs           # Clap argument definitions
    output.rs         # Terminal output formatting
  commands.rs         # Commands module root (future)
  commands/
    plan.rs           # Plan command implementation
    balance.rs        # Balance command implementation
    recurring.rs      # Recurring command implementation
```

**Key principle**: Use `module.rs` as the root that declares submodules, not `module/mod.rs` (old style).

### Development Guidelines
1. Keep commands fast (<50ms for most operations, especially `plan`)
2. Provide helpful error messages in Czech when appropriate
3. Support both Czech and English command names
4. Maintain backwards compatibility for data files
5. Write integration tests for CLI commands
6. **Use proper decimal arithmetic for money (never use f64)**
7. Implement proper transaction ordering (by date, then by creation time)
8. Support dry-run mode for destructive operations
9. Correctly handle month boundaries (31st → 28/29/30/31 depending on month)
10. Make balance setting flexible - any date, not just 13th of month
11. Follow modern module organization: `module.rs` + `module/submodule.rs`

### Learning Objectives (RON + SQLite Architecture)
This project is excellent for learning Rust because it covers:

1. **Serialization/Deserialization**
   - RON format with `serde` derives
   - Custom serialization for complex types (Decimal, DateTime)
   - Error handling during deserialization

2. **Async Programming**
   - Tokio runtime setup
   - Async file I/O operations
   - Async SQLite queries with `sqlx`
   - No blocking operations in async context

3. **Database Operations**
   - SQLite schema design and migrations
   - Prepared statements and query builders
   - Transaction management (ACID)
   - Index optimization for queries

4. **File System Operations**
   - File modification time (mtime) checking
   - Atomic file writes (write-then-rename pattern)
   - Directory creation and management
   - Home directory resolution (~/.cashflow/)

5. **Cache Invalidation**
   - Cache coherency strategies
   - Performance optimization (fast path vs slow path)
   - Metadata tracking

6. **CLI Design**
   - Clap derive macros and subcommands
   - Argument parsing and validation
   - Pretty-printing with tables
   - Error reporting to users

7. **Business Logic**
   - Date arithmetic with `chrono`
   - Decimal arithmetic for money
   - Projection algorithms (recurring transactions)
   - Balance calculation

8. **Rust Idioms**
   - Ownership and borrowing
   - Error propagation with `?` operator
   - Option and Result types
   - Pattern matching
   - Iterator chains for data transformation

### Key Problems Solved vs. Numbers Spreadsheet
1. ✅ No need to manually delete one-time transactions - automatic handling
2. ✅ Can set current balance on ANY date, not just 13th of month
3. ✅ Automatic projection of recurring payments without manual copying
4. ✅ Clear separation of recurring vs one-time transactions
5. ✅ Easy to disable/enable recurring payments without losing data
6. ✅ Fast lookups and projections without formula recalculation

### Future Enhancements
- **Phase 1 (MVP)**
  - ✅ Recurring transactions
  - ✅ One-time transactions
  - ✅ 30-day cashflow projection
  - ✅ Balance management

- **Phase 2**
  - Import from bank statements (CSV) - especially savings account exports
  - Automatic detection of actual payments vs projected
  - Reconciliation mode (compare actual vs planned)
  - Warning notifications when balance drops below threshold
  - Support for bi-weekly or custom recurring patterns

- **Phase 3**
  - Interactive TUI mode using `ratatui` for better UX
  - Charts and visualizations (balance over time)
  - Category-based analysis
  - Multi-currency support
  - Data sync across devices (optional cloud backup)
  - Mobile companion app (read-only view)

- **Nice to Have**
  - Integration with checking account (separate tracking)
  - Automatic balance updates via bank API
  - Smart suggestions for optimal transfer amounts
  - Budget alerts and forecasting
  - Export to formats compatible with accounting tools
