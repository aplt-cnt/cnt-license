use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;
use crate::license;

/// Executes the `source` command: outputs the raw content of a license.
///
/// Lookup order: built-in → custom.
/// If `year` or `author` are provided, `{year}` / `{author}` placeholders
/// are replaced in the output.
pub fn execute(license_name: &str, year: Option<&str>, author: Option<&str>, verbose: bool) -> Result<()> {
    if verbose {
        let licenses_dir = config::licenses_dir().unwrap_or_default();
        let is_builtin = license::is_builtin_license(license_name);
        println!("{} License: {}", "·".dimmed(), license_name.cyan());
        println!("{} Source: {}", "·".dimmed(), if is_builtin { "built-in".green().to_string() } else { format!("custom ({})", licenses_dir.join(license_name).display()).cyan().to_string() });
        println!("{} year: {}", "·".dimmed(), year.unwrap_or("(empty)").dimmed());
        println!("{} author: {}", "·".dimmed(), author.unwrap_or("(empty)").dimmed());
        println!();
    }

    let template = get_template(license_name)?;
    let year = year.unwrap_or("");
    let author = author.unwrap_or("");

    let output = license::generate_license(&template, year, author);

    if verbose {
        println!("{} Output: {} bytes\n", "·".dimmed(), output.len().to_string().yellow());
    }

    println!("{}", output);

    Ok(())
}

/// Retrieves the raw template for a license, looking up built-in first,
/// then falling back to custom.
fn get_template(license_name: &str) -> Result<String> {
    // Try built-in first
    if license::is_builtin_license(license_name) {
        return license::get_builtin_template(license_name).map(String::from);
    }

    // Try custom
    let licenses_dir = config::licenses_dir()?;
    let path = licenses_dir.join(license_name);
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow!("Failed to read custom license '{}': {}", license_name, e))?;
        return Ok(content);
    }

    Err(anyhow!(
        "License '{}' not found. Run 'clicense list' to see all installed licenses.",
        license_name
    ))
}
