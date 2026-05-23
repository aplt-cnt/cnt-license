use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;

use crate::config;
use crate::http;

/// Status of a single license template after comparing remote vs local.
enum ItemStatus {
    Added,
    Updated,
    Unchanged,
    LocalOnly,
}

/// A single diff result for one license template.
struct DiffItem {
    name: String,
    status: ItemStatus,
}

/// Executes `clicense update`: downloads the latest license templates
/// from the configured (or overridden) update_url, compares against local
/// files, and reports a detailed diff.
pub fn execute(override_url: Option<&str>) -> Result<()> {
    let base = http::resolve_url(override_url, None)?;

    println!(
        "{} Fetching license updates from {}...\n",
        "→".yellow(),
        base.cyan()
    );

    // --- HTTP fetch (YAML format) ---
    let url = format!("{}/api/v1/licenses", base);
    let body = http::get_yaml(&url)?;

    let remote: HashMap<String, String> = serde_yaml::from_str(&body)
        .map_err(|e| anyhow!("Failed to parse response as license templates: {}", e))?;

    // --- Scan local templates ---
    let licenses_dir = config::licenses_dir()?;
    let mut local: HashMap<String, String> = HashMap::new();
    if licenses_dir.exists() {
        for entry in fs::read_dir(&licenses_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if name.starts_with('.') {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    local.insert(name.to_string(), content);
                }
            }
        }
    }

    // --- Diff ---
    let mut diff_items: Vec<DiffItem> = Vec::new();

    for name in remote.keys() {
        let status = match local.get(name) {
            None => ItemStatus::Added,
            Some(local_content) => {
                if *local_content == remote[name] {
                    ItemStatus::Unchanged
                } else {
                    ItemStatus::Updated
                }
            }
        };
        diff_items.push(DiffItem {
            name: name.clone(),
            status,
        });
    }

    for name in local.keys() {
        if !remote.contains_key(name) {
            diff_items.push(DiffItem {
                name: name.clone(),
                status: ItemStatus::LocalOnly,
            });
        }
    }

    diff_items.sort_by(|a, b| a.name.cmp(&b.name));

    // --- Counters ---
    let mut added = 0u32;
    let mut updated = 0u32;
    let mut unchanged = 0u32;
    let mut local_only = 0u32;

    if diff_items.is_empty() {
        println!("  {} No templates found on server.", "—".dimmed());
        return Ok(());
    }

    // Header
    println!(
        "  {:<30} {}",
        "Name".bold().underline(),
        "Status".bold().underline()
    );
    println!("  {:-<30}", "");

    for item in &diff_items {
        let (icon, label, colorize): (&str, &str, fn(&str) -> colored::ColoredString) =
            match item.status {
                ItemStatus::Added => ("+", "ADDED", |s| s.green()),
                ItemStatus::Updated => ("~", "UPDATED", |s| s.yellow()),
                ItemStatus::Unchanged => ("=", "UNCHANGED", |s| s.dimmed()),
                ItemStatus::LocalOnly => ("!", "LOCAL ONLY", |s| s.red()),
            };

        println!(
            "  {} {:<28} {}",
            colorize(icon),
            colorize(&item.name),
            colorize(label)
        );

        match item.status {
            ItemStatus::Added => added += 1,
            ItemStatus::Updated => updated += 1,
            ItemStatus::Unchanged => unchanged += 1,
            ItemStatus::LocalOnly => local_only += 1,
        }
    }

    // --- Apply changes ---
    if added > 0 || updated > 0 {
        config::ensure_licenses_dir()?;
        for item in &diff_items {
            match item.status {
                ItemStatus::Added | ItemStatus::Updated => {
                    let path = licenses_dir.join(&item.name);
                    fs::write(&path, &remote[&item.name])?;
                }
                _ => {}
            }
        }
    }

    // --- Summary ---
    println!();
    println!("{}", "Summary:".bold());
    if added > 0 {
        println!(
            "  {} Added:    {}",
            "+".green(),
            added.to_string().cyan()
        );
    }
    if updated > 0 {
        println!(
            "  {} Updated:  {}",
            "~".yellow(),
            updated.to_string().cyan()
        );
    }
    if unchanged > 0 {
        println!(
            "  {} Unchanged: {}",
            "=".dimmed(),
            unchanged.to_string().dimmed()
        );
    }
    if local_only > 0 {
        println!(
            "  {} Local only: {} (not on server, kept as-is)",
            "!".red(),
            local_only.to_string().cyan()
        );
    }

    if added + updated == 0 && local_only == 0 {
        println!("\n  {} Everything is up-to-date.", "✓".green().bold());
    } else if added + updated > 0 {
        println!(
            "\n  {} {} license template(s) written to disk.",
            "✓".green().bold(),
            (added + updated).to_string().cyan()
        );
    }

    println!("\n{} Update complete.", "✔".green().bold());

    Ok(())
}
