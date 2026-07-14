use anyhow::Result;
use colored::Colorize;

use crate::config;

const BUILTIN_LICENSES: &[&str] = &[
    "mit",
    "apache-2.0",
    "gpl-3.0",
    "lgpl-3.0",
    "bsd-3-clause",
    "bsd-2-clause",
    "mpl-2.0",
    "unlicense",
    "isc",
    "epl-2.0",
];

fn get_builtin_content(id: &str) -> Option<&'static str> {
    match id {
        "mit" => Some(include_str!("../../../licenses/mit.txt")),
        "apache-2.0" => Some(include_str!("../../../licenses/apache-2.0.txt")),
        "gpl-3.0" => Some(include_str!("../../../licenses/gpl-3.0.txt")),
        "lgpl-3.0" => Some(include_str!("../../../licenses/lgpl-3.0.txt")),
        "bsd-3-clause" => Some(include_str!("../../../licenses/bsd-3-clause.txt")),
        "bsd-2-clause" => Some(include_str!("../../../licenses/bsd-2-clause.txt")),
        "mpl-2.0" => Some(include_str!("../../../licenses/mpl-2.0.txt")),
        "unlicense" => Some(include_str!("../../../licenses/unlicense.txt")),
        "isc" => Some(include_str!("../../../licenses/isc.txt")),
        "epl-2.0" => Some(include_str!("../../../licenses/epl-2.0.txt")),
        _ => None,
    }
}

/// Executes the `init` command: initializes server config and license templates.
pub fn execute(
    config_dir: Option<&str>,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
    force: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} force: {}", "·".dimmed(), force.to_string().dimmed());
        if let Some(d) = licenses_dir {
            println!("{} licenses_dir override: {}", "·".dimmed(), d.cyan());
        }
        if let Some(d) = meta_dir {
            println!("{} meta_dir override: {}", "·".dimmed(), d.cyan());
        }
        println!();
    }

    config::ServerConfig::ensure_config_dir(config_dir)?;

    let config_path = config::ServerConfig::config_file_path(config_dir)?;
    if !config_path.exists() {
        let defaults = config::ServerConfig::defaults();
        defaults.save_to_file(config_dir)?;
        println!(
            "{} Created config file: {}",
            "✓".green().bold(),
            config_path.display().to_string().cyan()
        );
    } else {
        println!(
            "{} Config file already exists: {}",
            "—".dimmed(),
            config_path.display().to_string().cyan()
        );
    }

    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved_licenses_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let resolved_meta_dir = config::resolve_meta_dir(meta_dir, &cfg);

    let dir = std::path::Path::new(&resolved_licenses_dir);
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create licenses directory '{}': {}",
                dir.display(),
                e
            )
        })?;
        println!(
            "{} Created licenses directory: {}",
            "✓".green().bold(),
            resolved_licenses_dir.cyan()
        );
    } else {
        println!(
            "{} Licenses directory already exists: {}",
            "—".dimmed(),
            resolved_licenses_dir.cyan()
        );
    }

    let meta_path = std::path::Path::new(&resolved_meta_dir);
    if !meta_path.exists() {
        std::fs::create_dir_all(meta_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create meta directory '{}': {}",
                meta_path.display(),
                e
            )
        })?;
        println!(
            "{} Created meta directory: {}",
            "✓".green().bold(),
            resolved_meta_dir.cyan()
        );
    } else {
        println!(
            "{} Meta directory already exists: {}",
            "—".dimmed(),
            resolved_meta_dir.cyan()
        );
    }

    let mut written = 0u32;
    let mut skipped = 0u32;

    for id in BUILTIN_LICENSES {
        let content = get_builtin_content(id).unwrap();
        let file_path = dir.join(format!("{}.txt", id));

        if file_path.exists() && !force {
            if verbose { println!("{} Skip: {}", "·".dimmed(), file_path.display().to_string().dimmed()); }
            println!(
                "  {} {} (already exists)",
                "⚠".yellow(),
                format!("{}.txt", id).cyan()
            );
            skipped += 1;
        } else {
            std::fs::write(&file_path, content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to write '{}': {}",
                    file_path.display(),
                    e
                )
            })?;
            if verbose { println!("{} Write: {} ({} bytes)", "·".dimmed(), file_path.display().to_string().dimmed(), content.len()); }
            println!(
                "  {} {} ({} bytes)",
                "✓".green().bold(),
                format!("{}.txt", id).cyan(),
                content.len().to_string().yellow()
            );
            written += 1;
        }
    }

    println!();
    println!(
        "{} Initialized {} license templates ({} written, {} skipped)",
        "✓".green().bold(),
        BUILTIN_LICENSES.len().to_string().cyan(),
        written.to_string().yellow(),
        skipped.to_string().yellow()
    );

    Ok(())
}
