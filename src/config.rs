use crate::Result;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs as async_fs;

/// Configuration stored in ~/.cashflowrc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashflowConfig {
    /// Path to data directory (where data.ron is stored)
    pub data_dir: PathBuf,
}

impl Default for CashflowConfig {
    fn default() -> Self {
        Self {
            data_dir: get_default_data_dir(),
        }
    }
}

/// Gets the default data directory based on environment and build type
/// Priority:
/// 1. CASHFLOW_DATA_DIR environment variable
/// 2. Debug builds: ./dev-data (local to repo, git-ignored)
/// 3. Release builds: ~/.cashflow
fn get_default_data_dir() -> PathBuf {
    // Check environment variable first
    if let Ok(env_dir) = std::env::var("CASHFLOW_DATA_DIR") {
        return PathBuf::from(env_dir);
    }

    // Use different directories for debug vs release builds
    #[cfg(debug_assertions)]
    {
        // Use local dev-data directory (git-ignored)
        PathBuf::from("./dev-data")
    }

    #[cfg(not(debug_assertions))]
    {
        let home = dirs::home_dir().expect("Failed to get home directory");
        home.join(".cashflow")
    }
}

/// Gets the path to the config file (~/.cashflowrc)
pub fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".cashflowrc"))
}

/// Loads config from ~/.cashflowrc
/// Returns default config if file doesn't exist
/// In debug builds, ignores .cashflowrc and uses local dev-data/ directory
pub async fn load_config() -> Result<CashflowConfig> {
    // In debug builds, skip .cashflowrc and use local dev directory
    #[cfg(debug_assertions)]
    {
        return Ok(CashflowConfig::default());
    }

    // In release builds, respect .cashflowrc
    #[cfg(not(debug_assertions))]
    {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(CashflowConfig::default());
        }

        let contents = async_fs::read_to_string(&config_path)
            .await
            .context("Failed to read config file")?;

        let config: CashflowConfig =
            toml::from_str(&contents).context("Failed to parse config file")?;

        Ok(config)
    }
}

/// Saves config to ~/.cashflowrc
pub async fn save_config(config: &CashflowConfig) -> Result<()> {
    let config_path = get_config_path()?;

    let contents = toml::to_string_pretty(config).context("Failed to serialize config")?;

    async_fs::write(&config_path, contents)
        .await
        .context("Failed to write config file")?;

    Ok(())
}

/// Gets the data directory path from config
/// Respects CASHFLOW_DATA_DIR environment variable as highest priority
pub async fn get_data_dir() -> Result<PathBuf> {
    // Check environment variable first (highest priority)
    if let Ok(env_dir) = std::env::var("CASHFLOW_DATA_DIR") {
        return Ok(PathBuf::from(env_dir));
    }

    let config = load_config().await?;
    Ok(config.data_dir)
}

/// Sets the data directory path in config
pub async fn set_data_dir(path: PathBuf) -> Result<()> {
    let mut config = load_config().await?;
    config.data_dir = path;
    save_config(&config).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CashflowConfig::default();

        // In debug builds (including tests), uses ./dev-data
        #[cfg(debug_assertions)]
        assert_eq!(config.data_dir, PathBuf::from("./dev-data"));

        // In release builds, uses ~/.cashflow
        #[cfg(not(debug_assertions))]
        assert!(config.data_dir.to_string_lossy().contains(".cashflow"));
    }

    #[test]
    fn test_serialize_config() {
        let config = CashflowConfig::default();
        let toml = toml::to_string_pretty(&config).unwrap();
        assert!(toml.contains("data_dir"));
    }
}
