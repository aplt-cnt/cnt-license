use std::path::PathBuf;

/// 服务器配置，从环境变量读取
pub struct Config {
    pub licenses_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let licenses_dir = std::env::var("LICENSES_DIR")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                // Try common relative paths
                for candidate in &["licenses", "../licenses", "../../licenses"] {
                    let p = PathBuf::from(candidate);
                    if p.exists() && p.is_dir() {
                        return p;
                    }
                }
                // Fallback: assume we're at workspace root
                PathBuf::from("licenses")
            });

        Self { licenses_dir }
    }
}
