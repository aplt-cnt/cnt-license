use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Returns the built-in license template for the given license ID.
/// Supported IDs: mit, apache-2.0, gpl-3.0, lgpl-3.0, bsd-3-clause,
/// bsd-2-clause, mpl-2.0, unlicense, isc, epl-2.0
pub fn get_builtin_template(license_id: &str) -> Result<&'static str> {
    let templates = builtin_templates();
    templates
        .get(license_id)
        .ok_or_else(|| anyhow!(
            "Unknown license: '{}'. Supported licenses: {}",
            license_id,
            supported_licenses().join(", ")
        ))
        .copied()
}

/// Returns a list of supported built-in license IDs.
pub fn supported_licenses() -> Vec<&'static str> {
    vec![
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
    ]
}

/// Checks whether a given license ID is a built-in license.
pub fn is_builtin_license(license_id: &str) -> bool {
    builtin_templates().contains_key(license_id)
}

/// Reads a custom license template from the licenses directory.
pub fn get_custom_template(license_id: &str) -> Result<String> {
    let licenses_dir = crate::config::licenses_dir()?;
    let path = licenses_dir.join(license_id);
    if !path.exists() {
        return Err(anyhow!("Custom license '{}' not found", license_id));
    }
    let content = fs::read_to_string(&path)?;
    Ok(content)
}

/// Generates a license by applying placeholder replacements to a template.
/// Placeholders: {year}, {author}
pub fn generate_license(template: &str, year: &str, author: &str) -> String {
    template
        .replace("{year}", year)
        .replace("{author}", author)
}

/// Writes the generated license content to a file.
pub fn write_license_file(output_path: &Path, content: &str) -> Result<()> {
    fs::write(output_path, content)?;
    Ok(())
}

/// All built-in license templates as a static HashMap.
fn builtin_templates() -> &'static HashMap<&'static str, &'static str> {
    use std::sync::LazyLock;
    static TEMPLATES: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("mit", include_str!("../licenses/mit.txt"));
        m.insert("apache-2.0", include_str!("../licenses/apache-2.0.txt"));
        m.insert("gpl-3.0", include_str!("../licenses/gpl-3.0.txt"));
        m.insert("lgpl-3.0", include_str!("../licenses/lgpl-3.0.txt"));
        m.insert("bsd-3-clause", include_str!("../licenses/bsd-3-clause.txt"));
        m.insert("bsd-2-clause", include_str!("../licenses/bsd-2-clause.txt"));
        m.insert("mpl-2.0", include_str!("../licenses/mpl-2.0.txt"));
        m.insert("unlicense", include_str!("../licenses/unlicense.txt"));
        m.insert("isc", include_str!("../licenses/isc.txt"));
        m.insert("epl-2.0", include_str!("../licenses/epl-2.0.txt"));
        m
    });
    &TEMPLATES
}
