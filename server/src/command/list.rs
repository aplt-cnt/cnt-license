use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `list` command: lists or displays license templates.
pub fn execute(
    config_dir: Option<&str>,
    name: Option<&str>,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved_licenses_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let resolved_meta_dir = config::resolve_meta_dir(meta_dir, &cfg);
    let dir = std::path::Path::new(&resolved_licenses_dir);
    let meta_path = std::path::Path::new(&resolved_meta_dir);

    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Licenses dir: {}", "·".dimmed(), resolved_licenses_dir.cyan());
        println!("{} Meta dir: {}", "·".dimmed(), resolved_meta_dir.cyan());
        println!("{} Dir exists: {}", "·".dimmed(), dir.exists().to_string().dimmed());
        println!();
    }

    if let Some(name) = name {
        return show_detail(dir, meta_path, name);
    }

    list_all(dir, meta_path)
}

fn list_all(dir: &std::path::Path, meta_path: &std::path::Path) -> Result<()> {
    if !dir.exists() {
        println!(
            "  {} No licenses directory found at '{}'",
            "—".dimmed(),
            dir.display()
        );
        println!(
            "  {} Run {} to initialize",
            "💡".yellow(),
            "clicense-server init".cyan()
        );
        return Ok(());
    }

    let meta_map = crate::data::load_meta(meta_path).unwrap_or_default();

    let mut entries: Vec<(String, u64)> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "txt") {
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            entries.push((id, size));
        }
    }

    if entries.is_empty() {
        println!(
            "  {} No license templates found in '{}'",
            "—".dimmed(),
            dir.display()
        );
        return Ok(());
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    println!(
        "{} License templates in {}",
        "⚙".yellow().bold(),
        dir.display().to_string().cyan()
    );
    println!();

    for (id, size) in &entries {
        let meta_info = meta_map.get(id).map(|m| m.name.as_str()).unwrap_or("—");
        println!(
            "  {:<16} {} — {}",
            id.cyan().bold(),
            format!("{} bytes", size).yellow(),
            meta_info.dimmed()
        );
    }

    println!();
    println!(
        "  {} total, use {} to see details",
        entries.len().to_string().cyan(),
        "clicense-server list <name>".cyan()
    );

    Ok(())
}

fn show_detail(dir: &std::path::Path, meta_path: &std::path::Path, name: &str) -> Result<()> {
    let file_path = dir.join(format!("{}.txt", name));

    if !file_path.exists() {
        return Err(anyhow!(
            "License '{}' not found in '{}'",
            name,
            dir.display()
        ));
    }

    let content = std::fs::read_to_string(&file_path).map_err(|e| {
        anyhow!("Failed to read '{}': {}", file_path.display(), e)
    })?;

    let meta_map = crate::data::load_meta(meta_path).unwrap_or_default();

    println!("{} {}", name.cyan().bold(), format!("({} bytes)", content.len()).yellow());
    println!();

    if let Some(meta) = meta_map.get(name) {
        println!("  {} {}", "Name:".dimmed(), meta.name.green());
        println!("  {} {}", "SPDX:".dimmed(), meta.spdx_id.green());
        println!("  {} {}", "Description:".dimmed(), meta.description);
        if !meta.permissions.is_empty() {
            println!("  {} {}", "Permissions:".dimmed(), meta.permissions.join(", "));
        }
        if !meta.conditions.is_empty() {
            println!("  {} {}", "Conditions:".dimmed(), meta.conditions.join(", "));
        }
        if !meta.limitations.is_empty() {
            println!("  {} {}", "Limitations:".dimmed(), meta.limitations.join(", "));
        }
        if !meta.custom.is_empty() {
            println!("  {} {:?}", "Custom:".dimmed(), meta.custom);
        }
        println!();
    }

    println!("{}", content);

    Ok(())
}
