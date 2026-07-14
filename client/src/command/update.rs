use std::io::Read;

use anyhow::{anyhow, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;

use crate::config;
use crate::http;

enum ItemStatus {
    Added,
    Updated,
    Unchanged,
    LocalOnly,
}

struct DiffItem {
    name: String,
    status: ItemStatus,
}

/// Executes `clicense update`: downloads the latest license templates and metadata
/// from the configured update_url via /api/v1/export (.zip), compares against
/// local files, and applies changes.
pub fn execute(override_url: Option<&str>, verbose: bool) -> Result<()> {
    let base = http::resolve_url(override_url, None)?;

    if verbose {
        let cfg = config::load_config().unwrap_or_default();
        let config_path = config::config_file_path().unwrap_or_default();
        let licenses_path = config::licenses_dir().unwrap_or_default();
        let meta_path = config::meta_dir().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} update_url (config): {}", "·".dimmed(), cfg.update_url.dimmed());
        println!("{} update_url (resolved): {}", "·".dimmed(), base.cyan());
        println!("{} Local licenses dir: {}", "·".dimmed(), licenses_path.display().to_string().dimmed());
        println!("{} Local meta dir: {}", "·".dimmed(), meta_path.display().to_string().dimmed());
        println!();
    }

    println!(
        "{} Fetching license updates from {}...\n",
        "→".yellow(),
        base.cyan()
    );

    let url = format!("{}/api/v1/export", base);
    if verbose {
        println!("{} GET {}", "·".dimmed(), url.cyan());
    }

    let response = ureq::get(&url).call().map_err(|e| {
        anyhow!("Network error connecting to '{}': {}", url, e)
    })?;

    if response.status() != 200 {
        return Err(anyhow!(
            "Server returned status {} for '{}'",
            response.status(),
            url
        ));
    }

    let mut zip_bytes = Vec::new();
    let mut body = response.into_body();
    body.read_to_vec().map(|v| zip_bytes = v).map_err(|e| {
        anyhow!("Failed to read response body from '{}': {}", url, e)
    })?;

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| anyhow!("Failed to read zip archive: {}", e))?;

    let mut remote_templates: HashMap<String, String> = HashMap::new();
    let mut remote_meta: HashMap<String, String> = HashMap::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();

        if let Some(relative) = entry_name.strip_prefix("licenses/") {
            if relative.ends_with(".txt") && !relative.contains('/') {
                let id = relative.strip_suffix(".txt").unwrap_or(relative).to_string();
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                remote_templates.insert(id, content);
            }
        } else if let Some(relative) = entry_name.strip_prefix("meta/") {
            if relative.ends_with(".meta.toml") && !relative.contains('/') {
                let id = relative.strip_suffix(".meta.toml").unwrap_or(relative).to_string();
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                remote_meta.insert(id, content);
            }
        }
    }

    if verbose {
        println!("{} Response: {} templates + {} meta files\n", "·".dimmed(),
            remote_templates.len().to_string().yellow(),
            remote_meta.len().to_string().yellow());
    }

    let licenses_dir = config::licenses_dir()?;
    let meta_dir = config::meta_dir()?;
    let mut local_templates: HashMap<String, String> = HashMap::new();

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
                    local_templates.insert(name.to_string(), content);
                }
            }
        }
    }

    let mut diff_items: Vec<DiffItem> = Vec::new();

    for name in remote_templates.keys() {
        let status = match local_templates.get(name) {
            None => ItemStatus::Added,
            Some(local_content) => {
                if *local_content == remote_templates[name] {
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

    for name in local_templates.keys() {
        if !remote_templates.contains_key(name) {
            diff_items.push(DiffItem {
                name: name.clone(),
                status: ItemStatus::LocalOnly,
            });
        }
    }

    diff_items.sort_by(|a, b| a.name.cmp(&b.name));

    let mut added = 0u32;
    let mut updated = 0u32;
    let mut unchanged = 0u32;
    let mut local_only = 0u32;

    if diff_items.is_empty() {
        println!("  {} No templates found on server.", "—".dimmed());
        return Ok(());
    }

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

    if added > 0 || updated > 0 {
        config::ensure_licenses_dir()?;
        config::ensure_meta_dir()?;

        for item in &diff_items {
            match item.status {
                ItemStatus::Added | ItemStatus::Updated => {
                    let path = licenses_dir.join(&item.name);
                    fs::write(&path, &remote_templates[&item.name])?;

                    if let Some(meta_content) = remote_meta.get(&item.name) {
                        let meta_path = meta_dir.join(format!("{}.meta.toml", item.name));
                        fs::write(&meta_path, meta_content)?;
                    }
                }
                _ => {}
            }
        }
    }

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
