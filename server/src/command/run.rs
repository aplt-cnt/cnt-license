use anyhow::{anyhow, Result};
use serde_json::json;

use crate::config;

/// 执行 `run` 子命令：启动 Axum API 服务器
///
/// 同步入口，内部创建 tokio Runtime 并 block_on
pub fn execute(host: Option<&str>, port: Option<u16>, licenses_dir: Option<&str>, verbose: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_run(host, port, licenses_dir, verbose))
}

/// 异步服务器启动逻辑（从原 main.rs 迁移）
async fn async_run(host: Option<&str>, port: Option<u16>, licenses_dir: Option<&str>, verbose: bool) -> Result<()> {
    use axum::{Router, routing::get};
    use colored::Colorize;
    use std::sync::Arc;
    use tower_http::cors::CorsLayer;
    use tower_http::trace::TraceLayer;

    // 1. 三级优先级 resolve
    let cfg = config::ServerConfig::load_from_file()?;
    let resolved = cfg.resolve(host, port, licenses_dir);

    if verbose {
        let config_path = config::ServerConfig::config_file_path().unwrap_or_default();
        println!("{} Config file: {}", "·".dimmed(), config_path.display().to_string().dimmed());
        println!("{} host (resolved): {}", "·".dimmed(), resolved.host.cyan());
        println!("{} port (resolved): {}", "·".dimmed(), resolved.port.to_string().cyan());
        println!("{} licenses_dir (resolved): {}", "·".dimmed(), resolved.licenses_dir.cyan());
        println!("{} log_level: {}", "·".dimmed(), resolved.log_level.dimmed());
        println!();
    }

    // 2. 初始化 tracing（使用配置中的 log_level）
    let env_filter = format!(
        "cnt_license_server={},tower_http={}",
        resolved.log_level, resolved.log_level
    );
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    // 3. 初始化应用状态
    let licenses_path = std::path::PathBuf::from(&resolved.licenses_dir);
    let app_state = crate::state::init_state(&licenses_path)
        .map_err(|e| anyhow!("Failed to load license data from '{}': {}", licenses_path.display(), e))?;
    let shared: crate::state::SharedState = Arc::new(app_state);

    // 4. 构建路由（与原 main.rs 完全一致）
    // --- /api/v1 — primary RESTful routes ---
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
        .with_state(shared.clone());

    // --- Backward-compatible aliases ---
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
        .with_state(shared.clone());

    // --- 404 fallback ---
    let not_found = Router::new().fallback(handler_404_not_found);

    let app = Router::new()
        .merge(compat)
        .nest("/api/v1", api_v1)
        .merge(not_found)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", resolved.host, resolved.port);
    tracing::info!("cnt-license-server listening on {}", addr);

    // Log the API structure
    tracing::info!("Routes:");
    tracing::info!("  GET /api/v1/health             — Health check");
    tracing::info!("  GET /api/v1/version            — Server version");
    tracing::info!("  GET /api/v1/licenses           — List all license templates");
    tracing::info!("  GET /api/v1/licenses/{{id}}     — Get one license template");
    tracing::info!("  GET /api/v1/licenses/{{id}}/info — Get license metadata");
    tracing::info!("  GET /api/v1/search?q=          — Search licenses");
    tracing::info!(
        "  (backward-compat aliases: /health, /licenses, /search, etc.)"
    );

    println!(
        "{} cnt-license-server listening on {}",
        "✓".green().bold(),
        addr.cyan()
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 404 handler — returns JSON error for unknown routes.
async fn handler_404_not_found() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        axum::Json(json!({"error": "Not found"})),
    )
}
