use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;

use crate::config;
use crate::license;

/// Executes the `new` command: generates a license file.
///
/// Resolution order for each parameter (CLI flag > config default > fallback):
/// - license_id: CLI arg > config default_license > error
/// - author:     CLI -a  > config default_author > error
/// - year:       CLI -y  > config default_year   > current year
/// - output:     CLI -o  > config output_name    > "LICENSE"
pub fn execute(license_id: Option<&str>, output: Option<&str>, year: Option<&str>, author: Option<&str>) -> Result<()> {
    // Load config for defaults
    let cfg = config::load_config().unwrap_or_default();

    // Resolve license_id: CLI > config > error
    let license_id = match license_id {
        Some(id) => id.to_string(),
        None => match &cfg.default_license {
            Some(dl) => dl.clone(),
            None => {
                return Err(anyhow!(
                    "License identifier is required. Use 'clicense new <license-id>' or set a default with 'clicense config default_license <id>'."
                ));
            }
        },
    };

    // Resolve author: CLI > config > error
    let author = match author {
        Some(a) => a.to_string(),
        None => match &cfg.default_author {
            Some(da) => da.clone(),
            None => {
                return Err(anyhow!(
                    "Author is required. Use -a or --author to specify, or set a default with 'clicense config default_author <name>'."
                ));
            }
        },
    };

    // Resolve year: CLI > config > current year
    let year = match year {
        Some(y) => y.to_string(),
        None => match &cfg.default_year {
            Some(dy) => dy.clone(),
            None => chrono::Local::now().format("%Y").to_string(),
        },
    };

    // Resolve output file name: CLI > config > "LICENSE"
    let output_name = match output {
        Some(o) => o.to_string(),
        None => cfg.output_name.clone(),
    };

    // Get the template (built-in or custom)
    let template = if license::is_builtin_license(&license_id) {
        license::get_builtin_template(&license_id)?.to_string()
    } else {
        // Try custom license
        match license::get_custom_template(&license_id) {
            Ok(t) => t,
            Err(_) => {
                return Err(anyhow!(
                    "License '{}' not found. Use 'clicense help' to see supported licenses, or add a custom license with 'clicense add'.",
                    license_id
                ));
            }
        }
    };

    // Generate the license content
    let content = license::generate_license(&template, &year, &author);

    // Write to file
    let output_path = Path::new(&output_name);
    license::write_license_file(output_path, &content)?;

    println!(
        "{} License '{}' generated successfully → {}",
        "✓".green().bold(),
        license_id.cyan(),
        output_name.yellow()
    );
    println!(
        "  {} Year: {}, Author: {}",
        "→".yellow(),
        year,
        author
    );

    Ok(())
}
