use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `config` command.
pub fn execute(
    config_dir: Option<&str>,
    key: Option<&str>,
    value: Option<&str>,
    list: bool,
    reset: Option<&str>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        let exists = config_path.exists();
        println!("{} Config file: {} ({})", "·".dimmed(),
            config_path.display().to_string().dimmed(),
            if exists { "exists".green().to_string() } else { "not found, using defaults".yellow().to_string() }
        );
        println!();
    }

    if list {
        return list_config(config_dir);
    }

    if let Some(reset_key) = reset {
        return reset_config(config_dir, reset_key, verbose);
    }

    let key = key.ok_or_else(|| {
        anyhow!(
            "Usage: clicense-server config <key> [value]\n       clicense-server config --list\n       clicense-server config --reset <key>"
        )
    })?;

    if let Some(val) = value {
        return set_config(config_dir, key, val, verbose);
    }

    show_config(config_dir, key)
}

fn list_config(config_dir: Option<&str>) -> Result<()> {
    let cfg = config::ServerConfig::load_from_file(config_dir)?;

    println!("{} Available configuration keys:", "⚙".yellow().bold());
    println!();

    for meta in config::config_keys() {
        let current_value = cfg.get_value(meta.key).unwrap_or_else(|| "(unknown)".to_string());
        let is_set = current_value != meta.default_value;

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

fn show_config(config_dir: Option<&str>, key: &str) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
            key
        ));
    }

    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let current = cfg.get_value(key).unwrap_or_else(|| "(unknown)".to_string());
    let meta = config::get_meta(key).unwrap();

    println!("  {} = {}", key.cyan().bold(), current.green());
    println!("  {} {}", "→".yellow(), meta.description);
    println!("  {} {}", "default:".dimmed(), meta.default_value.dimmed());

    Ok(())
}

fn set_config(config_dir: Option<&str>, key: &str, value: &str, verbose: bool) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
            key
        ));
    }

    let mut cfg = config::ServerConfig::load_from_file(config_dir)?;
    let old_value = cfg.get_value(key).unwrap_or_else(|| "(unknown)".to_string());
    cfg.set_value(key, value)?;
    cfg.save_to_file(config_dir)?;

    if verbose {
        let path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), path.display().to_string().dimmed());
        println!("{} Old value: {}", "·".dimmed(), old_value.dimmed());
        println!("{} New value: {}", "·".dimmed(), value.cyan());
        println!();
    }

    println!(
        "{} Config updated: {} = {}",
        "✓".green().bold(),
        key.cyan(),
        value.yellow()
    );

    Ok(())
}

fn reset_config(config_dir: Option<&str>, key: &str, verbose: bool) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
            key
        ));
    }

    let mut cfg = config::ServerConfig::load_from_file(config_dir)?;
    let old_value = cfg.get_value(key).unwrap_or_else(|| "(unknown)".to_string());
    cfg.reset_value(key)?;
    cfg.save_to_file(config_dir)?;

    if verbose {
        let path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), path.display().to_string().dimmed());
        println!("{} Old value: {}", "·".dimmed(), old_value.dimmed());
        println!();
    }

    println!(
        "{} Config key '{}' has been reset to default",
        "✓".green().bold(),
        key.cyan()
    );

    Ok(())
}
