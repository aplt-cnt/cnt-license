use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 服务器配置（持久化到 ~/.clicense-server/config.yml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_licenses_dir")]
    pub licenses_dir: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    3000
}
fn default_licenses_dir() -> String {
    "./licenses".to_string()
}
fn default_log_level() -> String {
    "info".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            licenses_dir: default_licenses_dir(),
            log_level: default_log_level(),
        }
    }
}

/// 配置键的元信息（用于 config 命令的展示与验证）
pub struct ConfigMeta {
    pub key: &'static str,
    pub description: &'static str,
    pub default_value: &'static str,
    pub value_type: &'static str,
}

/// 返回所有配置键的元信息
pub fn config_keys() -> Vec<ConfigMeta> {
    vec![
        ConfigMeta {
            key: "host",
            description: "服务器监听地址",
            default_value: "0.0.0.0",
            value_type: "String",
        },
        ConfigMeta {
            key: "port",
            description: "服务器监听端口",
            default_value: "3000",
            value_type: "u16",
        },
        ConfigMeta {
            key: "licenses_dir",
            description: "许可证模板目录",
            default_value: "./licenses",
            value_type: "String",
        },
        ConfigMeta {
            key: "log_level",
            description: "日志级别",
            default_value: "info",
            value_type: "String",
        },
    ]
}

/// 判断配置键是否有效
pub fn is_valid_key(key: &str) -> bool {
    config_keys().iter().any(|m| m.key == key)
}

/// 获取指定配置键的元信息
pub fn get_meta(key: &str) -> Option<ConfigMeta> {
    config_keys().into_iter().find(|m| m.key == key)
}

impl ServerConfig {
    /// 返回默认配置
    pub fn defaults() -> Self {
        Self::default()
    }

    /// 获取配置目录路径 (~/.clicense-server/)
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;
        Ok(home.join(".clicense-server"))
    }

    /// 获取配置文件路径 (~/.clicense-server/config.yml)
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.yml"))
    }

    /// 确保配置目录存在
    pub fn ensure_config_dir() -> Result<()> {
        let dir = Self::config_dir()?;
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(|e| {
                anyhow!(
                    "Failed to create config directory '{}': {}",
                    dir.display(),
                    e
                )
            })?;
        }
        Ok(())
    }

    /// 从配置文件加载，文件不存在则返回默认值
    pub fn load_from_file() -> Result<Self> {
        let path = Self::config_file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            anyhow!(
                "Failed to read config file '{}': {}",
                path.display(),
                e
            )
        })?;
        let cfg: ServerConfig = serde_yaml::from_str(&content).map_err(|e| {
            anyhow!(
                "Failed to parse config file '{}': {}",
                path.display(),
                e
            )
        })?;
        Ok(cfg)
    }

    /// 保存到配置文件
    pub fn save_to_file(&self) -> Result<()> {
        Self::ensure_config_dir()?;
        let path = Self::config_file_path()?;
        let content = serde_yaml::to_string(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, content).map_err(|e| {
            anyhow!(
                "Failed to write config file '{}': {}",
                path.display(),
                e
            )
        })?;
        Ok(())
    }

    /// 三级优先级合并：CLI 参数 > 配置文件 > 默认值
    ///
    /// 传入 CLI 参数的 Option 值，Some 则覆盖配置文件中的值
    pub fn resolve(&self, host: Option<&str>, port: Option<u16>, licenses_dir: Option<&str>) -> Self {
        Self {
            host: host
                .map(str::to_string)
                .unwrap_or_else(|| self.host.clone()),
            port: port.unwrap_or(self.port),
            licenses_dir: licenses_dir
                .map(str::to_string)
                .unwrap_or_else(|| self.licenses_dir.clone()),
            log_level: self.log_level.clone(),
        }
    }

    /// 获取指定配置键的当前值（字符串形式）
    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "host" => Some(self.host.clone()),
            "port" => Some(self.port.to_string()),
            "licenses_dir" => Some(self.licenses_dir.clone()),
            "log_level" => Some(self.log_level.clone()),
            _ => None,
        }
    }

    /// 设置指定配置键的值
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "host" => self.host = value.to_string(),
            "port" => {
                self.port = value.parse().map_err(|e| {
                    anyhow!("Invalid port value '{}': {}", value, e)
                })?;
            }
            "licenses_dir" => self.licenses_dir = value.to_string(),
            "log_level" => self.log_level = value.to_string(),
            _ => {
                return Err(anyhow!(
                    "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
                    key
                ));
            }
        }
        Ok(())
    }

    /// 将指定配置键重置为默认值
    pub fn reset_value(&mut self, key: &str) -> Result<()> {
        match key {
            "host" => self.host = default_host(),
            "port" => self.port = default_port(),
            "licenses_dir" => self.licenses_dir = default_licenses_dir(),
            "log_level" => self.log_level = default_log_level(),
            _ => {
                return Err(anyhow!(
                    "Unknown config key: '{}'. Run 'clicense-server config --list' to see all available keys.",
                    key
                ));
            }
        }
        Ok(())
    }
}

/// 解析许可证目录：CLI 覆盖 > 配置文件 > 默认值
pub fn resolve_licenses_dir(cli_override: Option<&str>, cfg: &ServerConfig) -> String {
    cli_override
        .map(str::to_string)
        .unwrap_or_else(|| cfg.licenses_dir.clone())
}
