use std::collections::HashMap;

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

fn parse_custom(custom: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in custom {
        let (k, v) = pair
            .split_once('=')
            .ok_or_else(|| anyhow!("Invalid custom key=value pair: '{}'", pair))?;
        map.insert(k.to_string(), v.to_string());
    }
    Ok(map)
}

/// Executes the `add` command: adds a custom license template with optional metadata.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    file: &str,
    name: &str,
    force: bool,
    display_name: Option<&str>,
    description: Option<&str>,
    spdx_id: Option<&str>,
    permissions: &[String],
    conditions: &[String],
    limitations: &[String],
    keywords: &[String],
    custom: &[String],
    verbose: bool,
) -> Result<()> {
    let content = std::fs::read_to_string(file).map_err(|e| {
        anyhow!("Failed to read file '{}': {}", file, e)
    })?;

    config::ensure_licenses_dir()?;
    config::ensure_meta_dir()?;

    let licenses_dir = config::licenses_dir()?;
    let meta_dir = config::meta_dir()?;
    let dest = licenses_dir.join(name);

    if verbose {
        let config_path = config::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Source file: {}", "·".dimmed(), file.cyan());
        println!("{} Source size: {} bytes", "·".dimmed(), content.len().to_string().yellow());
        println!("{} Destination: {}", "·".dimmed(), dest.display().to_string().cyan());
        println!("{} Meta dir: {}", "·".dimmed(), meta_dir.display().to_string().cyan());
        println!("{} Force overwrite: {}", "·".dimmed(), force.to_string().dimmed());
        println!();
    }

    if dest.exists() && !force {
        return Err(anyhow!(
            "A custom license named '{}' already exists. Use {} to overwrite.",
            name,
            format!("clicense add --name {} --force <file>", name).yellow()
        ));
    }

    let action = if dest.exists() { "updated" } else { "added" };
    std::fs::write(&dest, &content)?;

    let has_meta = display_name.is_some()
        || description.is_some()
        || spdx_id.is_some()
        || !permissions.is_empty()
        || !conditions.is_empty()
        || !limitations.is_empty()
        || !keywords.is_empty()
        || !custom.is_empty();

    if has_meta {
        let meta_content = toml::to_string_pretty(&toml::Value::Table({
            let mut table = toml::Table::new();
            if let Some(dn) = display_name {
                table.insert("name".to_string(), toml::Value::String(dn.to_string()));
            }
            if let Some(desc) = description {
                table.insert("description".to_string(), toml::Value::String(desc.to_string()));
            }
            if let Some(sid) = spdx_id {
                table.insert("spdx_id".to_string(), toml::Value::String(sid.to_string()));
            }
            if !permissions.is_empty() {
                table.insert("permissions".to_string(), toml::Value::Array(
                    permissions.iter().map(|p| toml::Value::String(p.clone())).collect()
                ));
            }
            if !conditions.is_empty() {
                table.insert("conditions".to_string(), toml::Value::Array(
                    conditions.iter().map(|c| toml::Value::String(c.clone())).collect()
                ));
            }
            if !limitations.is_empty() {
                table.insert("limitations".to_string(), toml::Value::Array(
                    limitations.iter().map(|l| toml::Value::String(l.clone())).collect()
                ));
            }
            if !keywords.is_empty() {
                table.insert("keywords".to_string(), toml::Value::Array(
                    keywords.iter().map(|k| toml::Value::String(k.clone())).collect()
                ));
            }
            if !custom.is_empty() {
                let custom_map = parse_custom(custom)?;
                let mut custom_table = toml::Table::new();
                for (k, v) in custom_map {
                    custom_table.insert(k, toml::Value::String(v));
                }
                table.insert("custom".to_string(), toml::Value::Table(custom_table));
            }
            table.insert("placeholders".to_string(), toml::Value::Array(Vec::new()));
            table
        })).map_err(|e| anyhow!("Failed to serialize metadata: {}", e))?;

        let meta_dest = meta_dir.join(format!("{}.meta.toml", name));
        std::fs::write(&meta_dest, &meta_content)?;
        if verbose {
            println!("{} Meta: {}", "·".dimmed(), meta_dest.display().to_string().cyan());
        }
    }

    println!(
        "{} Custom license '{}' {} successfully",
        "✓".green().bold(),
        name.cyan(),
        action
    );
    println!(
        "  {} Source: {}, Size: {} bytes",
        "→".yellow(),
        file,
        content.len().to_string().yellow()
    );

    Ok(())
}
