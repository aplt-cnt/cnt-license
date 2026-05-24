use std::collections::HashMap;

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `clone` command: clones license templates from a remote API server.
///
/// # Arguments
/// * `url`          - Remote server URL (e.g. http://localhost:3000)
/// * `licenses_dir` - CLI override for licenses directory
/// * `force`        - If true, overwrite existing license files
pub fn execute(url: &str, licenses_dir: Option<&str>, force: bool, verbose: bool) -> Result<()> {
    // 1. 解析许可证目录
    let cfg = config::ServerConfig::load_from_file()?;
    let resolved_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let dir = std::path::Path::new(&resolved_dir);

    // 2. 构建远程 URL
    let base_url = url.trim_end_matches('/');
    let api_url = format!("{}/api/v1/licenses", base_url);

    if verbose {
        let config_path = config::ServerConfig::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Remote URL: {}", "·".dimmed(), base_url.cyan());
        println!("{} GET {}", "·".dimmed(), api_url.cyan());
        println!("{} Licenses dir: {}", "·".dimmed(), resolved_dir.cyan());
        println!("{} force: {}", "·".dimmed(), force.to_string().dimmed());
        println!();
    }

    println!(
        "{} Cloning from {} ...",
        "→".yellow(),
        base_url.cyan()
    );

    // 3. 获取远程许可证数据
    let templates: HashMap<String, String> = crate::http::get_json(&api_url).map_err(|e| {
        anyhow!(
            "Failed to fetch licenses from '{}': {}",
            api_url, e
        )
    })?;

    if verbose {
        println!("{} Response: {} templates received\n", "·".dimmed(), templates.len().to_string().yellow());
    }

    if templates.is_empty() {
        println!(
            "  {} No license templates found at the remote server.",
            "—".dimmed()
        );
        return Ok(());
    }

    // 4. 确保许可证目录存在
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| {
            anyhow!(
                "Failed to create licenses directory '{}': {}",
                dir.display(),
                e
            )
        })?;
    }

    // 5. 写入模板文件
    let mut written = 0u32;
    let mut skipped = 0u32;

    for (id, content) in &templates {
        let file_path = dir.join(format!("{}.txt", id));

        if file_path.exists() && !force {
            println!(
                "  {} {} (already exists)",
                "⚠".yellow(),
                format!("{}.txt", id).cyan()
            );
            skipped += 1;
        } else {
            std::fs::write(&file_path, content).map_err(|e| {
                anyhow!("Failed to write '{}': {}", file_path.display(), e)
            })?;
            println!(
                "  {} {} ({} bytes)",
                "✓".green().bold(),
                format!("{}.txt", id).cyan(),
                content.len().to_string().yellow()
            );
            written += 1;
        }
    }

    // 6. 摘要
    println!();
    println!(
        "{} {} licenses fetched ({} written, {} skipped)",
        "✓".green().bold(),
        templates.len().to_string().cyan(),
        written.to_string().yellow(),
        skipped.to_string().yellow()
    );

    Ok(())
}
