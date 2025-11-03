use crate::Result;
use crate::config::{get_config_path, set_data_dir};
use std::path::PathBuf;

/// Executes config show command
pub async fn execute_config_show() -> Result<()> {
    let actual_data_dir = crate::config::get_data_dir().await?;

    println!("Configuration:");
    println!();

    // Show if environment variable is overriding
    if let Ok(env_dir) = std::env::var("CASHFLOW_DATA_DIR") {
        println!("Data directory: {} (from CASHFLOW_DATA_DIR)", env_dir);

        #[cfg(not(debug_assertions))]
        {
            use crate::config::load_config;

            let config_path = get_config_path()?;
            if config_path.exists() {
                let config = load_config().await?;
                println!(
                    "  Note: Config file {} exists but is overridden",
                    config_path.display()
                );
                println!("        Config value: {}", config.data_dir.display());
            }
        }
    } else {
        println!("Data directory: {}", actual_data_dir.display());

        // Show build type info for default directory
        #[cfg(debug_assertions)]
        {
            println!("  Source: Local development (./dev-data)");
            println!("  Note: .cashflowrc is ignored in development builds");
        }

        #[cfg(not(debug_assertions))]
        {
            let config_path = get_config_path()?;
            if config_path.exists() {
                println!("  Source: Config file ({})", config_path.display());
            } else {
                println!("  Source: Default (production mode → ~/.cashflow)");
            }
        }
    }

    println!();
    println!("💡 Tips:");

    #[cfg(debug_assertions)]
    {
        println!("  • Development uses local ./dev-data/ (git-ignored)");
        println!("  • Override with: export CASHFLOW_DATA_DIR=/path/to/data");
        println!("  • Your production .cashflowrc is ignored during development");
    }

    #[cfg(not(debug_assertions))]
    {
        let config_path = get_config_path()?;
        println!("  • Config file: {}", config_path.display());
        println!("  • Change directory: cashflow config set-data-dir <path>");
        println!("  • Override via env: export CASHFLOW_DATA_DIR=/path/to/data");
    }

    Ok(())
}

/// Executes config set-data-dir command
pub async fn execute_config_set_data_dir(path_str: &str) -> Result<()> {
    // Parse and expand path
    let path = if path_str.starts_with('~') {
        // Expand ~ to home directory
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        let path_without_tilde = path_str.strip_prefix("~/").unwrap_or(&path_str[1..]);
        home.join(path_without_tilde)
    } else {
        PathBuf::from(path_str)
    };

    // Convert to absolute path
    let absolute_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    };

    // Set the data directory
    set_data_dir(absolute_path.clone()).await?;

    println!("Data directory set to: {}", absolute_path.display());
    println!();
    println!("Configuration saved to: {}", get_config_path()?.display());

    Ok(())
}
