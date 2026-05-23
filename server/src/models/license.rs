use serde::{Deserialize, Serialize};

/// 许可证元信息（从 licenses.toml 加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseMeta {
    pub name: String,
    pub description: String,
    pub spdx_id: String,
    pub placeholders: Vec<String>,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub limitations: Vec<String>,
}

/// 搜索结果中的许可证条目
#[derive(Debug, Clone, Serialize)]
pub struct LicenseSearchEntry {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// 搜索响应
#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<LicenseSearchEntry>,
}

/// 错误响应
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
