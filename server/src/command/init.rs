use anyhow::Result;
use colored::Colorize;

use crate::config;

/// 内置许可证模板 ID 列表
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

/// 获取内置许可证模板内容
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

/// 执行 `init` 子命令：初始化服务器配置与许可证模板
///
/// # Arguments
/// * `licenses_dir` - CLI 覆盖的许可证目录路径
/// * `force` - 是否覆盖已存在的模板文件
pub fn execute(licenses_dir: Option<&str>, force: bool, verbose: bool) -> Result<()> {
    if verbose {
        let config_path = config::ServerConfig::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} force: {}", "·".dimmed(), force.to_string().dimmed());
        if let Some(d) = licenses_dir {
            println!("{} licenses_dir override: {}", "·".dimmed(), d.cyan());
        }
        println!();
    }

    // 1. 确保配置目录存在
    config::ServerConfig::ensure_config_dir()?;

    // 2. 创建默认配置文件（如不存在）
    let config_path = config::ServerConfig::config_file_path()?;
    if !config_path.exists() {
        let defaults = config::ServerConfig::defaults();
        defaults.save_to_file()?;
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

    // 3. 加载配置并解析许可证目录
    let cfg = config::ServerConfig::load_from_file()?;
    let resolved_dir = config::resolve_licenses_dir(licenses_dir, &cfg);

    // 4. 创建许可证目录
    let dir = std::path::Path::new(&resolved_dir);
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
            resolved_dir.cyan()
        );
    } else {
        println!(
            "{} Licenses directory already exists: {}",
            "—".dimmed(),
            resolved_dir.cyan()
        );
    }

    // 5. 写入内置许可证模板
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

    // 6. 摘要
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
