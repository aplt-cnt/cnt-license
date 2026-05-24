use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::config;

/// Executes the `add` command: adds a custom license template.
///
/// # Arguments
/// * `file`         - Path to the license template file
/// * `name`         - Name/identifier for the license
/// * `force`        - If true, overwrite existing license with the same name
/// * `licenses_dir` - CLI override for licenses directory
pub fn execute(file: &str, name: &str, force: bool, licenses_dir: Option<&str>, verbose: bool) -> Result<()> {
    // 1. 读取模板文件
    let content = std::fs::read_to_string(file).map_err(|e| {
        anyhow!("Failed to read file '{}': {}", file, e)
    })?;

    // 2. 解析许可证目录
    let cfg = config::ServerConfig::load_from_file()?;
    let resolved_dir = config::resolve_licenses_dir(licenses_dir, &cfg);
    let dir = std::path::Path::new(&resolved_dir);

    // 3. 确保许可证目录存在
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| {
            anyhow!(
                "Failed to create licenses directory '{}': {}",
                dir.display(),
                e
            )
        })?;
    }

    // 4. 写入许可证文件（licenses_dir/<name>.txt）
    let dest = dir.join(format!("{}.txt", name));

    if verbose {
        let config_path = config::ServerConfig::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} Source file: {}", "·".dimmed(), file.cyan());
        println!("{} Source size: {} bytes", "·".dimmed(), content.len().to_string().yellow());
        println!("{} Destination: {}", "·".dimmed(), dest.display().to_string().cyan());
        println!("{} force: {}", "·".dimmed(), force.to_string().dimmed());
        println!();
    }

    // 检查是否已存在
    if dest.exists() && !force {
        return Err(anyhow!(
            "A license named '{}' already exists. Use {} to overwrite.",
            name,
            format!("clicense-server add --name {} --force <file>", name).yellow()
        ));
    }

    let action = if dest.exists() { "updated" } else { "added" };
    std::fs::write(&dest, &content)?;

    println!(
        "{} License '{}' {} successfully",
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
