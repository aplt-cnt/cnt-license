use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `add` command: adds a custom license template.
///
/// # Arguments
/// * `file`   - Path to the license template file
/// * `name`   - Name/identifier for the custom license
/// * `force`  - If true, overwrite existing custom license with the same name
pub fn execute(file: &str, name: &str, force: bool) -> Result<()> {
    // Read the template file
    let content = std::fs::read_to_string(file).map_err(|e| {
        anyhow!("Failed to read file '{}': {}", file, e)
    })?;

    // Ensure licenses directory exists
    config::ensure_licenses_dir()?;

    // Write the custom license
    let licenses_dir = config::licenses_dir()?;
    let dest = licenses_dir.join(name);

    // Check if a license with this name already exists
    if dest.exists() && !force {
        return Err(anyhow!(
            "A custom license named '{}' already exists. Use {} to overwrite.",
            name,
            format!("clicense add --name {} --force <file>", name).yellow()
        ));
    }

    std::fs::write(&dest, &content)?;

    let action = if force && dest.exists() { "updated" } else { "added" };

    println!(
        "{} Custom license '{}' {} successfully",
        "✓".green().bold(),
        name.cyan(),
        action
    );
    println!(
        "  {} Source: {}, Size: {} bytes",
        "→".yellow(),
        file,
        content.len().to_string().yellow()
    );

    Ok(())
}
