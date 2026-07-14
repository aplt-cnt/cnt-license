use anyhow::{anyhow, Result};
use serde_json::json;

use crate::config;

/// Executes the `run` command: starts the Axum API server.
pub fn execute(
    config_dir: Option<&str>,
    host: Option<&str>,
    port: Option<u16>,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_run(config_dir, host, port, licenses_dir, meta_dir, verbose))
}

async fn async_run(
    config_dir: Option<&str>,
    host: Option<&str>,
    port: Option<u16>,
    licenses_dir: Option<&str>,
    meta_dir: Option<&str>,
    verbose: bool,
) -> Result<()> {
    use axum::{Router, routing::get};
    use colored::Colorize;
    use std::sync::Arc;
    use tower_http::cors::CorsLayer;

    let cfg = config::ServerConfig::load_from_file(config_dir)?;
    let resolved = cfg.resolve(host, port, licenses_dir, meta_dir);

    if verbose {
        let config_path = config::ServerConfig::config_file_path(config_dir).unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} host (resolved): {}", "·".dimmed(), resolved.host.cyan());
        println!("{} port (resolved): {}", "·".dimmed(), resolved.port.to_string().cyan());
        println!("{} licenses_dir (resolved): {}", "·".dimmed(), resolved.licenses_dir.cyan());
        println!("{} meta_dir (resolved): {}", "·".dimmed(), resolved.meta_dir.cyan());
        println!("{} log_level: {}", "·".dimmed(), resolved.log_level.dimmed());
        println!("{} access_log: {}", "·".dimmed(), resolved.access_log.to_string().cyan());
        println!();
    }

    let env_filter = format!(
        "cnt_license_server={}",
        resolved.log_level
    );
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    let licenses_path = std::path::PathBuf::from(&resolved.licenses_dir);
    let meta_path = std::path::PathBuf::from(&resolved.meta_dir);
    let app_state = crate::state::init_state(&licenses_path, &meta_path)
        .map_err(|e| anyhow!("Failed to load license data from '{}': {}", licenses_path.display(), e))?;
    let shared: crate::state::SharedState = Arc::new(app_state);

    let access_log_enabled = resolved.access_log;

    let api_v1 = Router::new()
        .route("/health", get(crate::handlers::health::health_check))
        .route("/version", get(crate::handlers::version::get_version))
        .route("/licenses", get(crate::handlers::licenses::list_all))
        .route("/licenses/{id}", get(crate::handlers::licenses::get_one))
        .route(
            "/licenses/{id}/info",
            get(crate::handlers::licenses::get_info),
        )
        .route("/search", get(crate::handlers::search::search))
        .route("/export", get(crate::handlers::export::export))
        .with_state(shared.clone());

    let compat = Router::new()
        .route("/", get(crate::handlers::licenses::list_all))
        .route("/health", get(crate::handlers::health::health_check))
        .route("/version", get(crate::handlers::version::get_version))
        .route("/licenses", get(crate::handlers::licenses::list_all))
        .route(
            "/licenses/{id}",
            get(crate::handlers::licenses::get_one),
        )
        .route(
            "/licenses/{id}/info",
            get(crate::handlers::licenses::get_info),
        )
        .route("/search", get(crate::handlers::search::search))
        .route("/export", get(crate::handlers::export::export))
        .with_state(shared.clone());

    let not_found = Router::new().fallback(handler_404_not_found);

    let mut app = Router::new()
        .merge(compat)
        .nest("/api/v1", api_v1)
        .merge(not_found)
        .layer(CorsLayer::permissive());

    if access_log_enabled {
        app = app.layer(axum::middleware::from_fn(access_log_middleware));
    }

    let addr = format!("{}:{}", resolved.host, resolved.port);
    tracing::info!("cnt-license-server listening on {}", addr);

    tracing::info!("Routes:");
    tracing::info!("  GET /api/v1/health             — Health check");
    tracing::info!("  GET /api/v1/version            — Server version");
    tracing::info!("  GET /api/v1/licenses           — List all license templates");
    tracing::info!("  GET /api/v1/licenses/{{id}}     — Get one license template");
    tracing::info!("  GET /api/v1/licenses/{{id}}/info — Get license metadata");
    tracing::info!("  GET /api/v1/search?q=          — Search licenses");
    tracing::info!("  GET /api/v1/export             — Export all licenses as .zip");

    println!(
        "{} cnt-license-server listening on {}",
        "✓".green().bold(),
        addr.cyan()
    );

    if access_log_enabled {
        println!("{} Access logging: {}", "·".dimmed(), "enabled".green());
    } else {
        println!("{} Access logging: {}", "·".dimmed(), "disabled".yellow());
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn access_log_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed();

    let status = response.status().as_u16();
    let latency_ms = elapsed.as_secs_f64() * 1000.0;

    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    let ts = now.format(&time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]").unwrap()).unwrap_or_default();
    println!(r#"[{}] "{} {}" {} {:.3}ms"#, ts, method, uri, status, latency_ms);

    response
}

async fn handler_404_not_found() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        axum::Json(json!({"error": "Not found"})),
    )
}
