use crate::Result;
use crate::config;
use crate::models::CashflowData;
use anyhow::Context;
use std::fs;
use std::path::PathBuf;
use tokio::fs as async_fs;

/// Gets the path to the Cashflow data directory (from config or default ~/.cashflow)
pub async fn get_data_dir() -> Result<PathBuf> {
    config::get_data_dir().await
}

/// Gets the path to the RON data file (data_dir/data.ron)
pub async fn get_ron_path() -> Result<PathBuf> {
    Ok(get_data_dir().await?.join("data.ron"))
}

/// Ensures the data directory exists
pub async fn ensure_data_dir() -> Result<PathBuf> {
    let dir = get_data_dir().await?;
    async_fs::create_dir_all(&dir)
        .await
        .context("Failed to create data directory")?;
    Ok(dir)
}

/// Loads CashflowData from the RON file
/// Returns empty data if file doesn't exist
pub async fn load_data() -> Result<CashflowData> {
    let ron_path = get_ron_path().await?;

    if !ron_path.exists() {
        // Return empty data on first run
        return Ok(CashflowData::default());
    }

    let contents = async_fs::read_to_string(&ron_path)
        .await
        .context("Failed to read data file")?;

    let data: CashflowData = ron::from_str(&contents).context("Failed to parse RON data file")?;

    Ok(data)
}

/// Saves CashflowData to the RON file atomically
/// Uses write-then-rename pattern to prevent corruption
pub async fn save_data(data: &CashflowData) -> Result<()> {
    ensure_data_dir().await?;

    let ron_path = get_ron_path().await?;
    let tmp_path = ron_path.with_extension("ron.tmp");

    // Serialize to RON with pretty formatting
    let ron_config = ron::ser::PrettyConfig::default()
        .depth_limit(4)
        .separate_tuple_members(true)
        .enumerate_arrays(false);

    let contents =
        ron::ser::to_string_pretty(data, ron_config).context("Failed to serialize data to RON")?;

    // Write to temporary file
    async_fs::write(&tmp_path, contents)
        .await
        .context("Failed to write temporary data file")?;

    // Atomic rename
    async_fs::rename(&tmp_path, &ron_path)
        .await
        .context("Failed to rename temporary file to data file")?;

    Ok(())
}

/// Gets the modification time of the RON file
/// Returns None if file doesn't exist
pub async fn get_ron_mtime() -> Result<Option<std::time::SystemTime>> {
    let ron_path = get_ron_path().await?;

    if !ron_path.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(&ron_path).context("Failed to get file metadata")?;

    let mtime = metadata
        .modified()
        .context("Failed to get file modification time")?;

    Ok(Some(mtime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BalanceSnapshot, OneTimeTransaction, RecurringTransaction};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_save_and_load_data() -> Result<()> {
        // Create test data
        let mut data = CashflowData::default();

        data.recurring.push(RecurringTransaction::new(
            "Netflix".to_string(),
            Decimal::from_str("-478").unwrap(),
            14,
        ));

        data.one_time.push(OneTimeTransaction::new(
            "Transfer from Air Bank".to_string(),
            Decimal::from_str("10000").unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 28).unwrap(),
        ));

        data.balance_snapshots.push(BalanceSnapshot::new(
            NaiveDate::from_ymd_opt(2025, 10, 13).unwrap(),
            Decimal::from_str("22158").unwrap(),
        ));

        // Save
        save_data(&data).await?;

        // Load
        let loaded = load_data().await?;

        // Verify
        assert_eq!(loaded.recurring.len(), 1);
        assert_eq!(loaded.recurring[0].description, "Netflix");
        assert_eq!(loaded.one_time.len(), 1);
        assert_eq!(loaded.balance_snapshots.len(), 1);

        Ok(())
    }
}
