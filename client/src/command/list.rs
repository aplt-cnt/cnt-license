use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;

use crate::config;
use crate::metadata::{self, LicenseMeta};

/// Executes the `list` command.
///
/// - No argument: lists all installed licenses (built-in + custom).
/// - `license_name`: shows detailed info for a specific license.
/// - `builtin` / `custom`: filter the listing.
pub fn execute(license_name: Option<&str>, builtin_only: bool, custom_only: bool, verbose: bool) -> Result<()> {
    if verbose {
        let config_path = config::config_file_path().unwrap_or_default();
        let licenses_path = config::licenses_dir().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Custom licenses dir: {}", "·".dimmed(), licenses_path.display().to_string().dimmed());
        println!("{} Filter: builtin_only={}, custom_only={}", "·".dimmed(), builtin_only, custom_only);
        println!();
    }
    match license_name {
        Some(name) => show_detail(name),
        None => list_all(builtin_only, custom_only),
    }
}

/// Lists installed licenses with optional filter.
fn list_all(builtin_only: bool, custom_only: bool) -> Result<()> {
    let builtin = metadata::builtin_ids();
    let custom = scan_custom()?;

    let show_builtin = !custom_only;
    let show_custom = !builtin_only;

    let total_builtin = if show_builtin { builtin.len() } else { 0 };
    let total_custom = if show_custom { custom.len() } else { 0 };

    println!(
        "{} Installed licenses ({} built-in + {} custom):\n",
        "📋".bold(),
        total_builtin.to_string().cyan(),
        total_custom.to_string().cyan()
    );

    // Built-in
    if show_builtin {
        println!("  {} Built-in:", "⚡".yellow());
        for id in &builtin {
            if let Some(meta) = metadata::get_meta(id) {
                println!("    {:<16} {}", id.cyan(), meta.name.dimmed());
            } else {
                println!("    {:<16}", id.cyan());
            }
        }
    }

    // Custom
    if show_custom && !custom.is_empty() {
        if show_builtin {
            println!();
        }
        println!("  {} Custom:", "📦".magenta());
        for name in &custom {
            let meta_name = metadata::get_meta_with_custom(name)
                .map(|m| m.name)
                .unwrap_or_else(|| "(custom)".to_string());
            println!("    {:<16} {}", name.yellow(), meta_name.dimmed());
        }
    }

    if !show_builtin && !show_custom {
        println!("  {} Use at most one of --builtin / --custom", "⚠".yellow());
        return Ok(());
    }

    println!(
        "\n{} Use {} for detailed info.",
        "💡".yellow(),
        "clicense list <name>".cyan()
    );

    Ok(())
}

/// Shows detailed info for a specific license.
fn show_detail(name: &str) -> Result<()> {
    let meta = metadata::get_meta_with_custom(name).ok_or_else(|| {
        // Check if it's a custom license
        if custom_exists(name) {
            anyhow!(
                "'{}' is a custom license with no metadata record. Use 'clicense source {}' to view its content.",
                name, name
            )
        } else {
            anyhow!(
                "Unknown license: '{}'. Run 'clicense list' to see all installed licenses.",
                name
            )
        }
    })?;

    print_detailed(name, &meta);
    Ok(())
}

/// Prints the full detailed view for a license.
pub fn print_detailed(_id: &str, meta: &LicenseMeta) {
    // Header: Name (SPDX ID)
    println!(
        "{} ({})\n",
        meta.name.bold(),
        meta.spdx_id.dimmed()
    );

    // Description
    println!("{}\n", meta.description.dimmed());

    // Permissions / Conditions / Limitations — 3 columns
    let col_width = 24;
    println!(
        "{:<col_width$} {:<col_width$} {}",
        format!("{}:", "Permissions").bold(),
        format!("{}:", "Conditions").bold(),
        format!("{}:", "Limitations").bold(),
    );

    let max_rows = meta
        .permissions
        .len()
        .max(meta.conditions.len())
        .max(meta.limitations.len());

    for i in 0..max_rows {
        let perm = meta
            .permissions
            .get(i)
            .map(|s| format!("  ✓ {}", s))
            .unwrap_or_default();
        let cond = meta
            .conditions
            .get(i)
            .map(|s| format!("  ⓘ {}", s))
            .unwrap_or_default();
        let lim = meta
            .limitations
            .get(i)
            .map(|s| format!("  ✕ {}", s))
            .unwrap_or_default();

        let perm_colored = perm.green();
        let cond_colored = cond.yellow();
        let lim_colored = lim.red();

        println!(
            "{:<col_width$} {:<col_width$} {}",
            perm_colored, cond_colored, lim_colored
        );
    }

    if !meta.custom.is_empty() {
        println!();
        println!("{}", "Custom fields:".bold());
        for (k, v) in &meta.custom {
            println!("  {}: {}", k.dimmed(), v);
        }
    }
}

fn scan_custom() -> Result<Vec<String>> {
    let licenses_dir = config::licenses_dir()?;
    if !licenses_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&licenses_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
            && !name.starts_with('.')
        {
            names.push(name.to_string());
        }
    }

    names.sort();
    Ok(names)
}

/// Checks if a custom license with the given name exists.
fn custom_exists(name: &str) -> bool {
    if let Ok(dir) = config::licenses_dir() {
        dir.join(name).exists()
    } else {
        false
    }
}
