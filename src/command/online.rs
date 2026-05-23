use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::Deserialize;

use crate::http;
use crate::metadata::LicenseMeta;

// --- Server response types (match cnt-license-server models) ---

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[allow(dead_code)]
    query: String,
    results: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize)]
struct SearchEntry {
    id: String,
    name: String,
    #[allow(dead_code)]
    description: String,
}

// --- Command implementations ---

/// `clicense online list` — fetches license list from server.
pub fn execute_list(override_url: Option<&str>) -> Result<()> {
    let base = http::resolve_url(override_url, None)?;
    let url = format!("{}/api/v1/search", base);

    println!(
        "{} Fetching license list from {}...\n",
        "→".yellow(),
        base.cyan()
    );

    let resp: SearchResponse = http::get_json(&url)?;

    if resp.results.is_empty() {
        println!("  {} No licenses found on the server.", "—".dimmed());
        return Ok(());
    }

    println!(
        "{} {} licenses available on the server:\n",
        "📋".bold(),
        resp.results.len().to_string().cyan()
    );

    for entry in &resp.results {
        println!(
            "  {:<20} {}",
            entry.id.cyan(),
            entry.name.dimmed()
        );
    }

    println!(
        "\n{} Use {} for detailed info about a specific license.",
        "💡".yellow(),
        "clicense online license <name>".cyan()
    );

    Ok(())
}

/// `clicense online license <name>` — detailed info from server.
pub fn execute_license(name: &str, override_url: Option<&str>) -> Result<()> {
    let base = http::resolve_url(override_url, None)?;
    let url = format!("{}/api/v1/licenses/{}/info", base, name);

    println!(
        "{} Fetching license info for '{}'...\n",
        "→".yellow(),
        name.cyan()
    );

    let meta: LicenseMeta = http::get_json(&url).map_err(|e| {
        anyhow!("License '{}' not found on server: {}", name, e)
    })?;

    crate::command::list::print_detailed(name, &meta);
    Ok(())
}

/// `clicense online source <name>` — raw license content from server.
pub fn execute_source(name: &str, override_url: Option<&str>) -> Result<()> {
    let base = http::resolve_url(override_url, None)?;
    let url = format!("{}/api/v1/licenses/{}", base, name);

    let body = http::get_raw(&url)?;

    // Server returns YAML by default: "{name}: |\n  content..."
    let content = extract_license_content(&body, name)?;
    println!("{}", content);
    Ok(())
}

// --- Helpers ---

/// Extracts the license template text from a YAML response.
/// The server returns format: `name: |\n  content...`
fn extract_license_content(yaml: &str, name: &str) -> Result<String> {
    let map: std::collections::HashMap<String, String> = serde_yaml::from_str(yaml)
        .map_err(|_| anyhow!("License '{}' not found on the server.", name))?;

    map.get(name)
        .cloned()
        .ok_or_else(|| anyhow!("License '{}' not found in server response.", name))
}
