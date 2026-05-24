use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `remove` command: removes one or more license templates.
///
/// # Arguments
/// * `names`        - List of license names to remove
/// * `all`          - If true, remove all license templates
/// * `licenses_dir` - CLI override for licenses directory
pub fn execute(names: &[String], all: bool, licenses_dir: Option<&str>, verbose: bool) -> Result<()> {
    // 解析许可证目录
    let cfg = config::ServerConfig::load_from_file()?;
    let resolved_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let dir = std::path::Path::new(&resolved_dir);

    if verbose {
        let config_path = config::ServerConfig::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Licenses dir: {}", "·".dimmed(), resolved_dir.cyan());
        println!("{} all: {}", "·".dimmed(), all.to_string().dimmed());
        if !names.is_empty() {
            println!("{} targets: {}", "·".dimmed(), names.join(", ").yellow());
        }
        println!();
    }

    if all {
        return remove_all(dir, verbose);
    }

    if names.is_empty() {
        return Err(anyhow!(
            "Usage: clicense-server remove <name> [<name>...]\n       clicense-server remove --all"
        ));
    }

    remove_names(dir, names, verbose)
}

/// 删除指定名称的许可证
fn remove_names(dir: &std::path::Path, names: &[String], verbose: bool) -> Result<()> {
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
            }
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

/// 删除所有许可证模板文件
fn remove_all(dir: &std::path::Path, verbose: bool) -> Result<()> {
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
            }
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

/// 打印删除摘要
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
