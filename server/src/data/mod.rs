use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::path::Path;

use zip::write::FileOptions;

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

/// 加载许可证元信息：编译时内置 + 运行时 meta_dir 中的 *.meta.toml
pub fn load_meta(meta_dir: &Path) -> Result<HashMap<String, LicenseMeta>, Box<dyn std::error::Error>> {
    let content = include_str!("licenses.toml");
    let mut meta: HashMap<String, LicenseMeta> = toml::from_str(content)?;

    if meta_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(meta_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml") {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Some(id) = file_stem.strip_suffix(".meta") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(custom_meta) = toml::from_str::<LicenseMeta>(&content) {
                                    meta.insert(id.to_string(), custom_meta);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(meta)
}

/// 构建导出 .zip 文件（供 CLI export 和 GET /api/v1/export 共用）
pub fn build_zip(
    templates: &HashMap<String, String>,
    meta: &HashMap<String, LicenseMeta>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Cursor::new(Vec::new());
    let mut zip_writer = zip::ZipWriter::new(&mut buf);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for (id, content) in templates {
        zip_writer.start_file(format!("licenses/{}.txt", id), options)?;
        zip_writer.write_all(content.as_bytes())?;
    }

    for (id, meta_item) in meta {
        let toml_str = toml::to_string_pretty(meta_item)?;
        zip_writer.start_file(format!("meta/{}.meta.toml", id), options)?;
        zip_writer.write_all(toml_str.as_bytes())?;
    }

    let finished = zip_writer.finish()?;
    let _ = finished;

    Ok(buf.into_inner())
}
