use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `export` command: exports all licenses to a .zip file.
pub fn execute(
    config_dir: Option<&str>,
    output: Option<&str>,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved_licenses_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let resolved_meta_dir = config::resolve_meta_dir(meta_dir, &cfg);
    let licenses_path = std::path::Path::new(&resolved_licenses_dir);
    let meta_path = std::path::Path::new(&resolved_meta_dir);

    let output_file = output.unwrap_or("clicense-export.zip");

    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Licenses dir: {}", "·".dimmed(), resolved_licenses_dir.cyan());
        println!("{} Meta dir: {}", "·".dimmed(), resolved_meta_dir.cyan());
        println!("{} Output: {}", "·".dimmed(), output_file.cyan());
        println!();
    }

    let templates = crate::data::load_templates(licenses_path)
        .map_err(|e| anyhow!("Failed to load templates: {}", e))?;
    let meta_map = crate::data::load_meta(meta_path)
        .map_err(|e| anyhow!("Failed to load metadata: {}", e))?;

    let zip_bytes = crate::data::build_zip(&templates, &meta_map)
        .map_err(|e| anyhow!("Failed to build zip: {}", e))?;

    std::fs::write(output_file, &zip_bytes)
        .map_err(|e| anyhow!("Failed to write '{}': {}", output_file, e))?;

    println!(
        "{} Exported {} templates + {} meta entries to {} ({} bytes)",
        "✓".green().bold(),
        templates.len().to_string().cyan(),
        meta_map.len().to_string().cyan(),
        output_file.cyan(),
        zip_bytes.len().to_string().yellow()
    );

    Ok(())
}
