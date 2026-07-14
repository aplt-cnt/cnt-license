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
pub fn execute(key: Option<&str>, value: Option<&str>, list: bool, reset: Option<&str>, verbose: bool) -> Result<()> {
    if verbose {
        let config_path = config::config_file_path().unwrap_or_default();
        let exists = config_path.exists();
        println!("{} Config file: {} ({})", "·".dimmed(),
            config_path.display().to_string().dimmed(),
            if exists { "exists".green().to_string() } else { "not found, using defaults".yellow().to_string() }
        );
        println!();
    }

    // --list mode
    if list {
        return list_config();
    }

    // --reset <key> mode
    if let Some(reset_key) = reset {
        return reset_config(reset_key, verbose);
    }

    // Need at least a key
    let key = key.ok_or_else(|| anyhow!(
        "Usage: clicense config <key> [value]\n       clicense config --list\n       clicense config --reset <key>"
    ))?;

    // config <key> <value> → set
    if let Some(val) = value {
        return set_config(key, val, verbose);
    }

    // config <key> → show
    show_config(key)
}

/// Lists all configurable keys with their current values and descriptions
fn list_config() -> Result<()> {
    let cfg = config::load_config()?;

    println!("{} Available configuration keys:", "⚙".yellow().bold());
    println!();

    for meta in config::config_keys() {
        let current_value = get_current_value(&cfg, meta.key);
        let is_set = current_value != meta.default_value;

        // Key name with type badge
        let key_display = if is_set {
            format!("{:<16} {}", meta.key.cyan().bold(), format!("[{}]", meta.value_type).dimmed())
        } else {
            format!("{:<16} {}", meta.key.normal(), format!("[{}]", meta.value_type).dimmed())
        };

        println!("  {}", key_display);
        println!("  {} {}", "→".yellow().dimmed(), meta.description);

        if is_set {
            println!("  {} {} {} {}", "current:".dimmed(), current_value.green(), "default:".dimmed(), meta.default_value.dimmed());
        } else {
            println!("  {} {}", "default:".dimmed(), meta.default_value.dimmed());
        }
        println!();
    }

    println!("  {} Use {} to set a value", "💡".yellow(), "clicense config <key> <value>".cyan());
    println!("  {} Use {} to reset to default", "💡".yellow(), "clicense config --reset <key>".cyan());

    Ok(())
}

/// Shows the current value of a specific config key
fn show_config(key: &str) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense config --list' to see all available keys.",
            key
        ));
    }

    let cfg = config::load_config()?;
    let current = get_current_value(&cfg, key);
    let meta = config::get_meta(key).unwrap();

    println!("  {} = {}", key.cyan().bold(), current.green());
    println!("  {} {}", "→".yellow(), meta.description);
    println!("  {} {}", "default:".dimmed(), meta.default_value.dimmed());

    Ok(())
}

/// Sets a configuration key to a given value
fn set_config(key: &str, value: &str, verbose: bool) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense config --list' to see all available keys.",
            key
        ));
    }

    let mut cfg = config::load_config()?;
    let old_value = get_current_value(&cfg, key);
    apply_config_value(&mut cfg, key, value)?;
    let path = config::config_file_path()?;
    config::save_config(&cfg)?;

    if verbose {
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

/// Resets a configuration key to its default value
fn reset_config(key: &str, verbose: bool) -> Result<()> {
    if !config::is_valid_key(key) {
        return Err(anyhow!(
            "Unknown config key: '{}'. Run 'clicense config --list' to see all available keys.",
            key
        ));
    }

    let mut cfg = config::load_config()?;
    let old_value = get_current_value(&cfg, key);
    reset_config_value(&mut cfg, key)?;
    let path = config::config_file_path()?;
    config::save_config(&cfg)?;

    if verbose {
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

/// Applies a value to the appropriate config field
fn apply_config_value(cfg: &mut config::AppConfig, key: &str, value: &str) -> Result<()> {
    match key {
        "update_url" => cfg.update_url = value.to_string(),
        "output_name" => cfg.output_name = value.to_string(),
        "default_author" => cfg.default_author = Some(value.to_string()),
        "default_year" => cfg.default_year = Some(value.to_string()),
        "default_license" => cfg.default_license = Some(value.to_string()),
        _ => unreachable!(), // already validated by is_valid_key
    }
    Ok(())
}

/// Resets a config key to its default value
fn reset_config_value(cfg: &mut config::AppConfig, key: &str) -> Result<()> {
    match key {
        "update_url" => cfg.update_url = "https://api.clicense.top".to_string(),
        "output_name" => cfg.output_name = "LICENSE".to_string(),
        "default_author" => cfg.default_author = None,
        "default_year" => cfg.default_year = None,
        "default_license" => cfg.default_license = None,
        _ => unreachable!(),
    }
    Ok(())
}

/// Gets the current display value for a config key
fn get_current_value(cfg: &config::AppConfig, key: &str) -> String {
    match key {
        "update_url" => cfg.update_url.clone(),
        "output_name" => cfg.output_name.clone(),
        "default_author" => cfg.default_author.clone().unwrap_or_else(|| "(not set)".to_string()),
        "default_year" => cfg.default_year.clone().unwrap_or_else(|| "(current year)".to_string()),
        "default_license" => cfg.default_license.clone().unwrap_or_else(|| "(not set)".to_string()),
        _ => "(unknown)".to_string(),
    }
}
