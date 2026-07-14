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
pub fn execute(license_id: Option<&str>, output: Option<&str>, year: Option<&str>, author: Option<&str>, verbose: bool) -> Result<()> {
    // Load config for defaults
    let cfg = config::load_config().unwrap_or_default();

    if verbose {
        let config_path = config::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Config values:", "·".dimmed());
        println!("    update_url    = {}", cfg.update_url.dimmed());
        println!("    output_name   = {}", cfg.output_name.dimmed());
        println!("    default_author= {}", cfg.default_author.as_deref().unwrap_or("(not set)").dimmed());
        println!("    default_year  = {}", cfg.default_year.as_deref().unwrap_or("(not set)").dimmed());
        println!("    default_license= {}", cfg.default_license.as_deref().unwrap_or("(not set)").dimmed());
        println!();
    }

    // Resolve license_id: CLI > config > error
    let license_id = match license_id {
        Some(id) => {
            if verbose { println!("{} license_id: {} (from CLI)", "·".dimmed(), id.cyan()); }
            id.to_string()
        }
        None => match &cfg.default_license {
            Some(dl) => {
                if verbose { println!("{} license_id: {} (from config default_license)", "·".dimmed(), dl.cyan()); }
                dl.clone()
            }
            None => {
                return Err(anyhow!(
                    "License identifier is required. Use 'clicense new <license-id>' or set a default with 'clicense config default_license <id>'."
                ));
            }
        },
    };

    // Resolve author: CLI > config > error
    let author = match author {
        Some(a) => {
            if verbose { println!("{} author: {} (from CLI)", "·".dimmed(), a.cyan()); }
            a.to_string()
        }
        None => match &cfg.default_author {
            Some(da) => {
                if verbose { println!("{} author: {} (from config default_author)", "·".dimmed(), da.cyan()); }
                da.clone()
            }
            None => {
                return Err(anyhow!(
                    "Author is required. Use -a or --author to specify, or set a default with 'clicense config default_author <name>'."
                ));
            }
        },
    };

    // Resolve year: CLI > config > current year
    let year = match year {
        Some(y) => {
            if verbose { println!("{} year: {} (from CLI)", "·".dimmed(), y.cyan()); }
            y.to_string()
        }
        None => match &cfg.default_year {
            Some(dy) => {
                if verbose { println!("{} year: {} (from config default_year)", "·".dimmed(), dy.cyan()); }
                dy.clone()
            }
            None => {
                let y = chrono::Local::now().format("%Y").to_string();
                if verbose { println!("{} year: {} (from system clock)", "·".dimmed(), y.cyan()); }
                y
            }
        },
    };

    // Resolve output file name: CLI > config > "LICENSE"
    let output_name = match output {
        Some(o) => {
            if verbose { println!("{} output: {} (from CLI)", "·".dimmed(), o.cyan()); }
            o.to_string()
        }
        None => {
            if verbose { println!("{} output: {} (from config output_name)", "·".dimmed(), cfg.output_name.cyan()); }
            cfg.output_name.clone()
        }
    };

    // Get the template (built-in or custom)
    let template = if license::is_builtin_license(&license_id) {
        if verbose { println!("{} template source: built-in\n", "·".dimmed()); }
        license::get_builtin_template(&license_id)?.to_string()
    } else {
        // Try custom license
        match license::get_custom_template(&license_id) {
            Ok(t) => {
                if verbose {
                    let dir = config::licenses_dir().unwrap_or_default();
                    println!("{} template source: custom ({})\n", "·".dimmed(), dir.join(&license_id).display());
                }
                t
            }
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

    if verbose {
        println!("{} Generated content: {} bytes", "·".dimmed(), content.len().to_string().yellow());
    }

    // Write to file
    let output_path = Path::new(&output_name);
    if verbose {
        println!("{} Writing to: {}\n", "·".dimmed(), output_path.display().to_string().cyan());
    }
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
