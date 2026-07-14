use std::io::{Cursor, Read};

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `import` command: imports licenses from a .zip or .toml file (local file only).
pub fn execute(
    config_dir: Option<&str>,
    file: &str,
    force: bool,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved_licenses_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let resolved_meta_dir = config::resolve_meta_dir(meta_dir, &cfg);
    let licenses_path = std::path::Path::new(&resolved_licenses_dir);
    let meta_path = std::path::Path::new(&resolved_meta_dir);

    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Import file: {}", "·".dimmed(), file.cyan());
        println!("{} Licenses dir: {}", "·".dimmed(), resolved_licenses_dir.cyan());
        println!("{} Meta dir: {}", "·".dimmed(), resolved_meta_dir.cyan());
        println!("{} force: {}", "·".dimmed(), force.to_string().dimmed());
        println!();
    }

    let path = std::path::Path::new(file);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "zip" => import_zip(file, licenses_path, meta_path, force, verbose),
        "toml" => import_toml(file, licenses_path, meta_path, force, verbose),
        _ => Err(anyhow!(
            "Unsupported format '{}': must be .zip or .toml",
            ext
        )),
    }
}

fn import_zip(
    file: &str,
    licenses_path: &std::path::Path,
    meta_path: &std::path::Path,
    force: bool,
    verbose: bool,
) -> Result<()> {
    let data = std::fs::read(file)
        .map_err(|e| anyhow!("Failed to read '{}': {}", file, e))?;
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| anyhow!("Failed to read zip archive: {}", e))?;

    if !licenses_path.exists() {
        std::fs::create_dir_all(licenses_path)?;
    }
    if !meta_path.exists() {
        std::fs::create_dir_all(meta_path)?;
    }

    let total = archive.len();
    let mut written_templates = 0u32;
    let mut written_meta = 0u32;
    let mut skipped = 0u32;

    for i in 0..total {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();

        if let Some(relative) = entry_name.strip_prefix("licenses/") {
            if relative.ends_with(".txt") && !relative.contains('/') {
                let dest = licenses_path.join(relative);
                if dest.exists() && !force {
                    if verbose { println!("{} Skip: {}", "·".dimmed(), relative); }
                    skipped += 1;
                    continue;
                }
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                std::fs::write(&dest, &content)?;
                println!("  {} {} ({} bytes)", "✓".green().bold(), relative.cyan(), content.len().to_string().yellow());
                written_templates += 1;
            }
        } else if let Some(relative) = entry_name.strip_prefix("meta/") {
            if relative.ends_with(".meta.toml") && !relative.contains('/') {
                let dest = meta_path.join(relative);
                if dest.exists() && !force {
                    continue;
                }
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                std::fs::write(&dest, &content)?;
                if verbose { println!("{} Meta: {}", "·".dimmed(), relative); }
                written_meta += 1;
            }
        }
    }

    println!();
    println!(
        "{} Imported {} templates + {} meta entries ({} skipped)",
        "✓".green().bold(),
        written_templates.to_string().cyan(),
        written_meta.to_string().cyan(),
        skipped.to_string().yellow()
    );

    Ok(())
}

fn import_toml(
    _file: &str,
    _licenses_path: &std::path::Path,
    _meta_path: &std::path::Path,
    _force: bool,
    _verbose: bool,
) -> Result<()> {
    Err(anyhow!("TOML import format is not yet supported. Use .zip format instead."))
}
