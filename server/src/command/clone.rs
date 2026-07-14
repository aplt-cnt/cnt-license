use std::io::Read;

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `clone` command: clones license templates from a remote API server via /api/v1/export.
pub fn execute(
    config_dir: Option<&str>,
    url: &str,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
    force: bool,
    verbose: bool,
) -> Result<()> {
    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved_licenses_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let resolved_meta_dir = config::resolve_meta_dir(meta_dir, &cfg);
    let licenses_path = std::path::Path::new(&resolved_licenses_dir);
    let meta_path = std::path::Path::new(&resolved_meta_dir);

    let base_url = url.trim_end_matches('/');
    let api_url = format!("{}/api/v1/export", base_url);

    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Remote URL: {}", "·".dimmed(), base_url.cyan());
        println!("{} GET {}", "·".dimmed(), api_url.cyan());
        println!("{} Licenses dir: {}", "·".dimmed(), resolved_licenses_dir.cyan());
        println!("{} Meta dir: {}", "·".dimmed(), resolved_meta_dir.cyan());
        println!("{} force: {}", "·".dimmed(), force.to_string().dimmed());
        println!();
    }

    println!(
        "{} Cloning from {} ...",
        "→".yellow(),
        base_url.cyan()
    );

    let response = ureq::get(&api_url).call().map_err(|e| {
        anyhow!("Failed to fetch export from '{}': {}", api_url, e)
    })?;

    if response.status() != 200 {
        return Err(anyhow!(
            "Server returned status {} for '{}'",
            response.status(),
            api_url
        ));
    }

    let mut zip_bytes = Vec::new();
    let mut body = response.into_body();
    body.read_to_vec().map(|v| zip_bytes = v).map_err(|e| {
        anyhow!("Failed to read response body from '{}': {}", api_url, e)
    })?;

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| anyhow!("Failed to read zip archive: {}", e))?;

    if !licenses_path.exists() {
        std::fs::create_dir_all(licenses_path).map_err(|e| {
            anyhow!("Failed to create licenses directory '{}': {}", licenses_path.display(), e)
        })?;
    }
    if !meta_path.exists() {
        std::fs::create_dir_all(meta_path).map_err(|e| {
            anyhow!("Failed to create meta directory '{}': {}", meta_path.display(), e)
        })?;
    }

    let mut written = 0u32;
    let mut skipped = 0u32;
    let total = archive.len();

    for i in 0..total {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();

        if let Some(relative) = entry_name.strip_prefix("licenses/") {
            if relative.ends_with(".txt") && !relative.contains('/') {
                let dest = licenses_path.join(relative);
                if dest.exists() && !force {
                    println!("  {} {} (already exists)", "⚠".yellow(), relative.cyan());
                    skipped += 1;
                    continue;
                }
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                std::fs::write(&dest, &content)?;
                println!("  {} {} ({} bytes)", "✓".green().bold(), relative.cyan(), content.len().to_string().yellow());
                written += 1;
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
            }
        }
    }

    println!();
    println!(
        "{} {} entries in archive ({} templates written, {} skipped)",
        "✓".green().bold(),
        total.to_string().cyan(),
        written.to_string().yellow(),
        skipped.to_string().yellow()
    );

    Ok(())
}
