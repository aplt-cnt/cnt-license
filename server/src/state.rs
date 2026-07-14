use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::models::license::LicenseMeta;

/// Application global state
pub struct AppState {
    pub templates: HashMap<String, String>,
    pub meta: HashMap<String, LicenseMeta>,
    pub version: String,
}

pub type SharedState = Arc<AppState>;

/// Initialize AppState: load templates from disk + merge built-in & custom metadata.
pub fn init_state(
    licenses_dir: &Path,
    meta_dir: &Path,
) -> Result<AppState, Box<dyn std::error::Error>> {
    let templates = crate::data::load_templates(licenses_dir)?;
    let meta = crate::data::load_meta(meta_dir)?;
    Ok(AppState {
        templates,
        meta,
        version: "1.1.0".to_string(),
    })
}
