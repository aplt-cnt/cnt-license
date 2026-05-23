use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Application configuration stored in ~/.clicense/config.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Remote URL for downloading license template updates
    #[serde(default = "default_update_url")]
    pub update_url: String,
    /// Default output file name when generating licenses
    #[serde(default = "default_output_name")]
    pub output_name: String,
    /// Default copyright holder name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_author: Option<String>,
    /// Default copyright year
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_year: Option<String>,
    /// Default license identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_license: Option<String>,
}

fn default_update_url() -> String {
    "https://api.clicense.top".to_string()
}

fn default_output_name() -> String {
    "LICENSE".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            update_url: default_update_url(),
            output_name: default_output_name(),
            default_author: None,
            default_year: None,
            default_license: None,
        }
    }
}

/// Metadata for a config key (for display purposes)
#[derive(Debug, Clone)]
pub struct ConfigMeta {
    pub key: &'static str,
    pub description: &'static str,
    pub default_value: &'static str,
    pub value_type: &'static str,
}

/// Static list of all configurable keys with metadata
static CONFIG_KEYS: LazyLock<Vec<ConfigMeta>> = LazyLock::new(|| {
    vec![
        ConfigMeta {
            key: "update_url",
            description: "Remote URL for downloading license template updates",
            default_value: "https://api.clicense.top",
            value_type: "string",
        },
        ConfigMeta {
            key: "output_name",
            description: "Default output file name when generating licenses",
            default_value: "LICENSE",
            value_type: "string",
        },
        ConfigMeta {
            key: "default_author",
            description: "Default copyright holder name (used when -a is omitted)",
            default_value: "(not set)",
            value_type: "string",
        },
        ConfigMeta {
            key: "default_year",
            description: "Default copyright year (used when -y is omitted)",
            default_value: "(current year)",
            value_type: "string",
        },
        ConfigMeta {
            key: "default_license",
            description: "Default license identifier (used with 'clicense new' without ID)",
            default_value: "(not set)",
            value_type: "string",
        },
    ]
});

/// Returns the list of all configurable keys with metadata
pub fn config_keys() -> &'static Vec<ConfigMeta> {
    &CONFIG_KEYS
}

/// Checks whether a config key is valid
pub fn is_valid_key(key: &str) -> bool {
    CONFIG_KEYS.iter().any(|m| m.key == key)
}

/// Returns the ConfigMeta for a given key, if it exists
pub fn get_meta(key: &str) -> Option<&'static ConfigMeta> {
    CONFIG_KEYS.iter().find(|m| m.key == key)
}

/// Returns the path to the clicense config directory (~/.clicense/)
pub fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    Ok(home.join(".clicense"))
}

/// Returns the path to the config file (~/.clicense/config.yml)
pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.yml"))
}

/// Returns the path to the custom licenses directory (~/.clicense/licenses/)
pub fn licenses_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("licenses"))
}

/// Ensures that the config directory exists, creating it if necessary.
pub fn ensure_config_dir() -> Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Ensures that the licenses directory exists, creating it if necessary.
pub fn ensure_licenses_dir() -> Result<()> {
    let dir = licenses_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// Loads the configuration from disk. Returns default config if file doesn't exist.
pub fn load_config() -> Result<AppConfig> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: AppConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Saves the configuration to disk.
pub fn save_config(config: &AppConfig) -> Result<()> {
    ensure_config_dir()?;
    let path = config_file_path()?;
    let content = serde_yaml::to_string(config)?;
    fs::write(&path, content)?;
    Ok(())
}
