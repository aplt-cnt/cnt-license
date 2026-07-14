use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `source` command: outputs raw license template content.
pub fn execute(
    config_dir: Option<&str>,
    name: &str,
    licenses_dir: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved_licenses_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let dir = std::path::Path::new(&resolved_licenses_dir);

    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Licenses dir: {}", "·".dimmed(), resolved_licenses_dir.cyan());
        println!();
    }

    let file_path = dir.join(format!("{}.txt", name));
    if !file_path.exists() {
        return Err(anyhow!(
            "License '{}' not found in '{}'",
            name,
            dir.display()
        ));
    }

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| anyhow!("Failed to read '{}': {}", file_path.display(), e))?;

    println!("{}", content);

    Ok(())
}
