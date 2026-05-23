use std::collections::HashMap;
use std::sync::LazyLock;

/// License metadata loaded from the embedded licenses-meta.toml.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LicenseMeta {
    pub name: String,
    pub spdx_id: String,
    pub description: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub placeholders: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub limitations: Vec<String>,
}

/// Static cache of all built-in license metadata.
static META: LazyLock<HashMap<String, LicenseMeta>> = LazyLock::new(|| {
    let raw: HashMap<String, toml::Value> =
        toml::from_str(include_str!("../licenses-meta.toml"))
            .expect("Failed to parse licenses-meta.toml");

    raw.into_iter()
        .map(|(id, val)| {
            let meta = LicenseMeta {
                name: val["name"].as_str().unwrap_or(&id).to_string(),
                spdx_id: val["spdx_id"].as_str().unwrap_or(&id).to_string(),
                description: val["description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                placeholders: extract_strings(&val, "placeholders"),
                keywords: extract_strings(&val, "keywords"),
                permissions: extract_strings(&val, "permissions"),
                conditions: extract_strings(&val, "conditions"),
                limitations: extract_strings(&val, "limitations"),
            };
            (id, meta)
        })
        .collect()
});

/// Extracts a Vec<String> from a TOML array field.
fn extract_strings(val: &toml::Value, field: &str) -> Vec<String> {
    val.get(field)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Looks up metadata for a specific license id.
pub fn get_meta(id: &str) -> Option<&'static LicenseMeta> {
    META.get(id)
}

/// Returns all built-in license IDs sorted alphabetically.
pub fn builtin_ids() -> Vec<&'static str> {
    let mut ids: Vec<&str> = META.keys().map(|s| s.as_str()).collect();
    ids.sort();
    ids
}
