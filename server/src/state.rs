use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::models::license::LicenseMeta;

/// 应用全局状态
pub struct AppState {
    pub templates: HashMap<String, String>,
    pub meta: HashMap<String, LicenseMeta>,
    pub version: String,
}

pub type SharedState = Arc<AppState>;

/// 初始化 AppState：从磁盘加载模板 + 编译时嵌入元信息
pub fn init_state(licenses_dir: &Path) -> Result<AppState, Box<dyn std::error::Error>> {
    let templates = crate::data::load_templates(licenses_dir)?;
    let meta = crate::data::load_meta()?;
    Ok(AppState {
        templates,
        meta,
        version: "0.1.0".to_string(),
    })
}
