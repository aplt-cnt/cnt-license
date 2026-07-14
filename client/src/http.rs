//! Shared HTTP client for cnt-license CLI.
//!
//! Provides a configured `ureq::Agent` with:
//! - 15-second connect + receive-body timeouts
//! - User-Agent header identifying the client
//! - Convenience methods for JSON / raw / YAML responses
//! - HTTP error handling with informative messages

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::config;

const TIMEOUT: Duration = Duration::from_secs(15);
const USER_AGENT: &str = concat!("cnt-license/", env!("CARGO_PKG_VERSION"));

/// Builds a reusable `ureq::Agent` with timeouts.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(TIMEOUT))
        .timeout_recv_body(Some(TIMEOUT))
        .build()
        .into()
}

/// Resolves the server base URL: CLI override > config file.
pub fn resolve_url(override_url: Option<&str>, _config_key: Option<&str>) -> Result<String> {
    if let Some(u) = override_url {
        return Ok(u.trim_end_matches('/').to_string());
    }
    let cfg = config::load_config().unwrap_or_default();
    Ok(cfg.update_url.trim_end_matches('/').to_string())
}

// ---------------------------------------------------------------------------
// Convenience methods
// ---------------------------------------------------------------------------

/// HTTP GET → deserialized JSON value.
pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T> {
    let body = raw_get(url, Some("application/json"))?;
    serde_json::from_str(&body)
        .map_err(|e| anyhow!("Failed to parse JSON from '{}': {}", url, e))
}

/// HTTP GET → raw string body with optional Accept header.
pub fn get_raw(url: &str) -> Result<String> {
    raw_get(url, None)
}

/// HTTP GET → raw body, expecting YAML content type.
pub fn get_yaml(url: &str) -> Result<String> {
    raw_get(url, Some("application/yaml"))
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

/// Core GET helper: sends a request, handles errors, reads body.
fn raw_get(url: &str, accept: Option<&str>) -> Result<String> {
    let mut req = agent()
        .get(url)
        .header("User-Agent", USER_AGENT);
    if let Some(ct) = accept {
        req = req.header("Accept", ct);
    }

    let response = req.call().map_err(|e| {
        // ureq 3.x Error wraps Transport errors directly
        let msg = e.to_string();
        anyhow!(
            "Network error connecting to '{}': {}\n  (Check that the server is running and the URL is correct.)",
            url, msg
        )
    })?;

    response
        .into_body()
        .read_to_string()
        .map_err(|e| anyhow!("Failed to read response body from '{}': {}", url, e))
}
