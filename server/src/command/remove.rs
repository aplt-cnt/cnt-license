use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `remove` command: removes one or more license templates and their metadata.
pub fn execute(
    config_dir: Option<&str>,
    names: &[String],
    all: bool,
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
        println!("{} all: {}", "·".dimmed(), all.to_string().dimmed());
        if !names.is_empty() {
            println!("{} targets: {}", "·".dimmed(), names.join(", ").yellow());
        }
        println!();
    }

    if all {
        return remove_all(dir, meta_path, verbose);
    }

    if names.is_empty() {
        return Err(anyhow!(
            "Usage: clicense-server remove <name> [<name>...]\n       clicense-server remove --all"
        ));
    }

    remove_names(dir, meta_path, names, verbose)
}

fn remove_names(dir: &std::path::Path, meta_path: &std::path::Path, names: &[String], verbose: bool) -> Result<()> {
    if !dir.exists() {
        return Err(anyhow!("Licenses directory '{}' not found.", dir.display()));
    }

    let mut removed = 0u32;
    let mut not_found = Vec::new();
    let mut errors = Vec::new();

    for name in names {
        let path = dir.join(format!("{}.txt", name));
        if !path.exists() {
            not_found.push(name.as_str());
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => {
                if verbose { println!("{} Deleted: {}", "·".dimmed(), path.display().to_string().dimmed()); }
                println!(
                    "  {} Removed '{}'",
                    "✓".green().bold(),
                    name.cyan()
                );
                removed += 1;
            }
            Err(e) => {
                errors.push((name.as_str(), e));
                continue;
            }
        }

        let meta_file = meta_path.join(format!("{}.meta.toml", name));
        if meta_file.exists() {
            let _ = std::fs::remove_file(&meta_file);
            if verbose { println!("{} Deleted meta: {}", "·".dimmed(), meta_file.display().to_string().dimmed()); }
        }
    }

    print_summary(removed, &not_found, &errors)?;

    if !errors.is_empty() {
        return Err(anyhow!(
            "{} errors occurred during removal.",
            errors.len()
        ));
    }

    Ok(())
}

fn remove_all(dir: &std::path::Path, meta_path: &std::path::Path, verbose: bool) -> Result<()> {
    if !dir.exists() {
        println!(
            "  {} No licenses to remove.",
            "—".dimmed()
        );
        return Ok(());
    }

    let entries: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".txt") && !n.starts_with('.'))
        .collect();

    if entries.is_empty() {
        println!(
            "  {} No licenses to remove.",
            "—".dimmed()
        );
        return Ok(());
    }

    let count = entries.len();
    println!(
        "{} Removing all {} license(s)...\n",
        "→".yellow(),
        count.to_string().cyan()
    );

    let mut removed = 0u32;
    let mut errors = Vec::new();

    for name in &entries {
        let path = dir.join(name);
        match std::fs::remove_file(&path) {
            Ok(()) => {
                if verbose { println!("{} Deleted: {}", "·".dimmed(), path.display().to_string().dimmed()); }
                println!(
                    "  {} Removed '{}'",
                    "✓".green().bold(),
                    name.cyan()
                );
                removed += 1;
            }
            Err(e) => {
                errors.push((name.as_str(), e));
                continue;
            }
        }

        let base = name.strip_suffix(".txt").unwrap_or(name);
        let meta_file = meta_path.join(format!("{}.meta.toml", base));
        if meta_file.exists() {
            let _ = std::fs::remove_file(&meta_file);
            if verbose { println!("{} Deleted meta: {}", "·".dimmed(), meta_file.display().to_string().dimmed()); }
        }
    }

    println!();
    if removed > 0 {
        println!(
            "{} {} license(s) removed.",
            "✓".green().bold(),
            removed.to_string().cyan()
        );
    }
    if !errors.is_empty() {
        for (name, e) in &errors {
            println!(
                "  {} Failed to remove '{}': {}",
                "✗".red(),
                name.cyan(),
                e
            );
        }
        return Err(anyhow!(
            "{} errors occurred during removal.",
            errors.len()
        ));
    }

    Ok(())
}

fn print_summary(
    removed: u32,
    not_found: &[&str],
    _errors: &[(&str, std::io::Error)],
) -> Result<()> {
    println!();

    if removed > 0 {
        println!(
            "{} {} license(s) removed.",
            "✓".green().bold(),
            removed.to_string().cyan()
        );
    }
    if !not_found.is_empty() {
        println!(
            "{} {} license(s) not found.",
            "!".yellow(),
            not_found.len().to_string().cyan()
        );
        for name in not_found {
            println!("    {} — {}", name.cyan(), "not found".dimmed());
        }
    }

    Ok(())
}
