use std::collections::HashMap;

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;
use crate::models::license::LicenseMeta;

/// Parses --custom key=value pairs into a HashMap.
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
    config_dir: Option<&str>,
    file: &str,
    name: &str,
    force: bool,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
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

    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved_licenses_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let resolved_meta_dir = config::resolve_meta_dir(meta_dir, &cfg);
    let licenses_path = std::path::Path::new(&resolved_licenses_dir);
    let meta_path = std::path::Path::new(&resolved_meta_dir);

    if !licenses_path.exists() {
        std::fs::create_dir_all(licenses_path).map_err(|e| {
            anyhow!(
                "Failed to create licenses directory '{}': {}",
                licenses_path.display(),
                e
            )
        })?;
    }

    let dest = licenses_path.join(format!("{}.txt", name));

    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Source file: {}", "·".dimmed(), file.cyan());
        println!("{} Source size: {} bytes", "·".dimmed(), content.len().to_string().yellow());
        println!("{} Destination: {}", "·".dimmed(), dest.display().to_string().cyan());
        println!("{} Meta dir: {}", "·".dimmed(), meta_path.display().to_string().cyan());
        println!("{} force: {}", "·".dimmed(), force.to_string().dimmed());
        println!();
    }

    if dest.exists() && !force {
        return Err(anyhow!(
            "A license named '{}' already exists. Use {} to overwrite.",
            name,
            format!("clicense-server add --name {} --force <file>", name).yellow()
        ));
    }

    let action = if dest.exists() { "updated" } else { "added" };
    std::fs::write(&dest, &content)?;

    if !meta_path.exists() {
        std::fs::create_dir_all(meta_path).map_err(|e| {
            anyhow!(
                "Failed to create meta directory '{}': {}",
                meta_path.display(),
                e
            )
        })?;
    }

    let meta = LicenseMeta {
        name: display_name.unwrap_or(name).to_string(),
        description: description.unwrap_or("").to_string(),
        spdx_id: spdx_id.unwrap_or(name).to_string(),
        placeholders: Vec::new(),
        keywords: keywords.iter().cloned().collect(),
        permissions: permissions.iter().cloned().collect(),
        conditions: conditions.iter().cloned().collect(),
        limitations: limitations.iter().cloned().collect(),
        custom: parse_custom(custom)?,
    };

    let meta_dest = meta_path.join(format!("{}.meta.toml", name));
    let meta_content = toml::to_string_pretty(&meta)
        .map_err(|e| anyhow!("Failed to serialize metadata: {}", e))?;
    std::fs::write(&meta_dest, &meta_content)?;

    println!(
        "{} License '{}' {} successfully",
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
    println!(
        "  {} Meta: {}",
        "→".yellow(),
        meta_dest.display().to_string().cyan()
    );

    Ok(())
}
