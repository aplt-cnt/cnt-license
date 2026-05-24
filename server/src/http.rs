//! Shared HTTP client for cnt-license-server.
//!
//! Provides a configured `ureq::Agent` with:
//! - 15-second connect + receive-body timeouts
//! - User-Agent header identifying the server
//! - Convenience methods for JSON / raw responses
//! - HTTP error handling with informative messages

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(15);
const USER_AGENT: &str = concat!("clicense-server/", env!("CARGO_PKG_VERSION"));

/// Builds a reusable `ureq::Agent` with timeouts.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(TIMEOUT))
        .timeout_recv_body(Some(TIMEOUT))
        .build()
        .into()
}

/// HTTP GET → deserialized JSON value.
pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T> {
    let body = raw_get(url, Some("application/json"))?;
    serde_json::from_str(&body)
        .map_err(|e| anyhow!("Failed to parse JSON from '{}': {}", url, e))
}

/// HTTP GET → raw string body.
#[allow(dead_code)]
pub fn get_raw(url: &str) -> Result<String> {
    raw_get(url, None)
}

/// Core GET helper: sends a request, handles errors, reads body.
fn raw_get(url: &str, accept: Option<&str>) -> Result<String> {
    let mut req = agent()
        .get(url)
        .header("User-Agent", USER_AGENT);
    if let Some(ct) = accept {
        req = req.header("Accept", ct);
    }

    let response = req.call().map_err(|e| {
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
