use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes `clicense remove`: removes one or more custom license templates.
///
/// Each name is processed independently — a failure on one does not
/// prevent the others from being removed.
pub fn execute(names: &[String], verbose: bool) -> Result<()> {
    if names.is_empty() {
        return Err(anyhow!("Usage: clicense remove <name> [<name>...]"));
    }

    let licenses_dir = config::licenses_dir()?;

    if !licenses_dir.exists() {
        return Err(anyhow!("No custom licenses found."));
    }

    if verbose {
        let config_path = config::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Licenses dir: {}", "·".dimmed(), licenses_dir.display().to_string().cyan());
        println!("{} Targets: {}", "·".dimmed(), names.join(", ").yellow());
        println!();
    }

    let mut removed = 0u32;
    let mut not_found = Vec::new();
    let mut errors = Vec::new();
    let meta_dir = config::meta_dir().unwrap_or_default();

    for name in names {
        let path = licenses_dir.join(name);
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

        let meta_file = meta_dir.join(format!("{}.meta.toml", name));
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

/// Executes `clicense remove --all`: removes ALL custom license templates.
pub fn execute_all(verbose: bool) -> Result<()> {
    let licenses_dir = config::licenses_dir()?;

    if !licenses_dir.exists() {
        println!("  {} No custom licenses to remove.", "—".dimmed());
        return Ok(());
    }

    if verbose {
        let config_path = config::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Licenses dir: {}", "·".dimmed(), licenses_dir.display().to_string().cyan());
        println!();
    }

    let entries: Vec<String> = std::fs::read_dir(&licenses_dir)?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| !n.starts_with('.'))
        .collect();

    if entries.is_empty() {
        println!("  {} No custom licenses to remove.", "—".dimmed());
        return Ok(());
    }

    let count = entries.len();
    println!(
        "{} Removing all {} custom license(s)...\n",
        "→".yellow(),
        count.to_string().cyan()
    );

    let mut removed = 0u32;
    let mut errors = Vec::new();
    let meta_dir = config::meta_dir().unwrap_or_default();

    for name in &entries {
        let path = licenses_dir.join(name);
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

        let meta_file = meta_dir.join(format!("{}.meta.toml", name));
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
                "✕".red(),
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

/// Prints the summary for `remove`.
fn print_summary(removed: u32, not_found: &[&str], _errors: &[(&str, std::io::Error)]) -> Result<()> {
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
