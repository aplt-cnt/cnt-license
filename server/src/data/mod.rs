use std::collections::HashMap;
use std::path::Path;

use crate::models::license::LicenseMeta;

/// 从磁盘加载许可证模板文件
pub fn load_templates(licenses_dir: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use std::fs;

    if !licenses_dir.exists() {
        return Err(format!("Licenses directory '{}' not found", licenses_dir.display()).into());
    }

    let mut templates = HashMap::new();
    for entry in fs::read_dir(licenses_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "txt") {
            let license_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let content = fs::read_to_string(&path)?;
            templates.insert(license_id, content);
        }
    }
    Ok(templates)
}

/// 编译时从 licenses.toml 加载许可证元信息
pub fn load_meta() -> Result<HashMap<String, LicenseMeta>, Box<dyn std::error::Error>> {
    let content = include_str!("licenses.toml");
    let meta: HashMap<String, LicenseMeta> = toml::from_str(content)?;
    Ok(meta)
}
