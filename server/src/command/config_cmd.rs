use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `config` command.
///
/// Modes:
/// - `config --list`         → show all configurable keys and their current values
/// - `config <key> <value>`  → set a configuration key
/// - `config <key>`          → show the current value of a specific key
/// - `config --reset <key>`  → reset a key to its default value
pub fn execute(key: Option<&str>, value: Option<&str>, list: bool, reset: Option<&str>) -> Result<()> {
    // --list mode
    if list {
        return list_config();
    }

    // --reset <key> mode
    if let Some(reset_key) = reset {
        return reset_config(reset_key);
    }

    // Need at least a key
    let key = key.ok_or_else(|| {
        anyhow!(
            "Usage: clicense-server config <key> [value]\n       clicense-server config --list\n       clicense-server config --reset <key>"
        )
    })?;

    // config <key> <value> → set
    if let Some(val) = value {
        return set_config(key, val);
    }

    // config <key> → show
    show_config(key)
}

/// Lists all configurable keys with their current values and descriptions
fn list_config() -> Result<()> {
    let cfg = config::ServerConfig::load_from_file()?;

    println!("{} Available configuration keys:", "⚙".yellow().bold());
    println!();

    for meta in config::config_keys() {
        let current_value = cfg.get_value(meta.key).unwrap_or_else(|| "(unknown)".to_string());
        let is_set = current_value != meta.default_value;

        // Key name with type badge
        let key_display = if is_set {
            format!(
                "{:<16} {}",
                meta.key.cyan().bold(),
                format!("[{}]", meta.value_type).dimmed()
            )
        } else {
            format!(
                "{:<16} {}",
                meta.key.normal(),
                format!("[{}]", meta.value_type).dimmed()
            )
        };

        println!("  {}", key_display);
        println!("  {} {}", "→".yellow().dimmed(), meta.description);

        if is_set {
            println!(
                "  {} {} {} {}",
                "current:".dimmed(),
                current_value.green(),
                "default:".dimmed(),
                meta.default_value.dimmed()
            );
        } else {
            println!(
                "  {} {}",
                "default:".dimmed(),
                meta.default_value.dimmed()
            );
        }
        println!();
    }

    println!(
        "  {} Use {} to set a value",
        "💡".yellow(),
        "clicense-server config <key> <value>".cyan()
    );
    println!(
        "  {} Use {} to reset to default",
        "💡".yellow(),
        "clicense-server config --reset <key>".cyan()
    );

    Ok(())
}

/// Shows the current value of a specific config key
fn show_config(key: &str) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
            key
        ));
    }

    let cfg = config::ServerConfig::load_from_file()?;
    let current = cfg.get_value(key).unwrap_or_else(|| "(unknown)".to_string());
    let meta = config::get_meta(key).unwrap();

    println!("  {} = {}", key.cyan().bold(), current.green());
    println!("  {} {}", "→".yellow(), meta.description);
    println!("  {} {}", "default:".dimmed(), meta.default_value.dimmed());

    Ok(())
}

/// Sets a configuration key to a given value
fn set_config(key: &str, value: &str) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
            key
        ));
    }

    let mut cfg = config::ServerConfig::load_from_file()?;
    cfg.set_value(key, value)?;
    cfg.save_to_file()?;

    println!(
        "{} Config updated: {} = {}",
        "✓".green().bold(),
        key.cyan(),
        value.yellow()
    );

    Ok(())
}

/// Resets a configuration key to its default value
fn reset_config(key: &str) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
            key
        ));
    }

    let mut cfg = config::ServerConfig::load_from_file()?;
    cfg.reset_value(key)?;
    cfg.save_to_file()?;

    println!(
        "{} Config key '{}' has been reset to default",
        "✓".green().bold(),
        key.cyan()
    );

    Ok(())
}
