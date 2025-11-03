# Development Setup & Data Safety

## Problem Solved

✅ **Your production data is now safe!** Development builds (`cargo run`) automatically use a local `./dev-data/` directory and **completely ignore** your `.cashflowrc` file.

## How It Works

**Development Mode** (`cargo run`):
- Uses `./dev-data/` directory (local to repo)
- **Ignores** `~/.cashflowrc` completely
- Directory is git-ignored (won't be committed)
- Your production data stays untouched!

**Production Mode** (release binary):
- Respects `~/.cashflowrc` configuration
- Uses your custom directory or `~/.cashflow/` by default

## Data Directory Priority

1. **`CASHFLOW_DATA_DIR` environment variable** (highest priority - overrides everything)
2. **Build type**:
   - Debug: `./dev-data/` (ignores .cashflowrc)
   - Release: Uses `~/.cashflowrc` if exists, otherwise `~/.cashflow/`

## Quick Reference

### Check Current Configuration

```bash
# See which directory is being used
cargo run -- config show

# Or in production
cashflow config show
```

### Development Workflow

```bash
# Development - automatically uses ./dev-data/
cargo run -- plan
cargo run -- balance set 10000
cargo run -- recurring add -d "Test" --amount=-100 --day=15

# Your production data remains untouched!
# .cashflowrc is completely ignored during development
```

### Using Environment Variable Override

```bash
# Use a temporary test directory
CASHFLOW_DATA_DIR=/tmp/test-data cargo run -- plan

# Or set it for your shell session
export CASHFLOW_DATA_DIR=/tmp/test-data
cargo run -- plan
cargo run -- balance set 5000
```

### Switch Between Data Directories

```bash
# Use production data temporarily in development
CASHFLOW_DATA_DIR=~/.cashflow cargo run -- plan

# Use dev data in production binary (testing)
CASHFLOW_DATA_DIR=~/.cashflow-dev cashflow plan

# Use your custom iCloud directory
CASHFLOW_DATA_DIR=~/iCloud/Dokumenty/Výdaje cargo run -- plan
```

## Your Current Setup

You have a **custom data directory** set in `~/.cashflowrc`:
- **Production config**: `~/iCloud/Dokumenty/Výdaje`

### Good News - No Changes Needed!

✅ **Just use `cargo run` for development** - it automatically uses `./dev-data/` and ignores your `.cashflowrc`

✅ **Use the release binary for production** - it respects your custom config

**That's it!** No aliases, no environment variables needed for normal development.

## Accessing Your Production Data in Development

Your production data is safely stored in: `~/iCloud/Dokumenty/Výdaje`

To test with your real data during development:
```bash
# Use environment variable to override
CASHFLOW_DATA_DIR=~/iCloud/Dokumenty/Výdaje cargo run -- plan
```

**Warning**: Be careful when testing with production data!

## Safety Tips

1. ✅ Always check which directory you're using: `cashflow config show`
2. ✅ Test changes in dev directory first
3. ✅ Backup your data before making major changes
4. ✅ Your production data has `.ron` extension, easy to backup: `cp ~/iCloud/Dokumenty/Výdaje/data.ron ~/backup/`

## Example Workflow

```bash
# 1. Development - test new features safely (uses ./dev-data automatically)
cargo run -- balance set 1000
cargo run -- recurring add -d "Test" --amount=-50 --day=10
cargo run -- plan

# 2. Build release and use production data
cargo build --release
./target/release/cashflow plan  # Uses your .cashflowrc config

# 3. Or install the binary
sudo cp target/release/cashflow /usr/local/bin/
cashflow plan  # Production ready!
```

