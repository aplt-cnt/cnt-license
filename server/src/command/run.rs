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
        println!("{} access_log: {}", "·".dimmed(), resolved.access_log.to_string().cyan());
        println!();
    }

    // 2. 初始化 tracing（使用配置中的 log_level）
    let env_filter = format!(
        "cnt_license_server={}",
        resolved.log_level
    );
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    // 3. 初始化应用状态
    let licenses_path = std::path::PathBuf::from(&resolved.licenses_dir);
    let app_state = crate::state::init_state(&licenses_path)
        .map_err(|e| anyhow!("Failed to load license data from '{}': {}", licenses_path.display(), e))?;
    let shared: crate::state::SharedState = Arc::new(app_state);

    let access_log_enabled = resolved.access_log;

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

    if access_log_enabled {
        println!("{} Access logging: {}", "·".dimmed(), "enabled".green());
    } else {
        println!("{} Access logging: {}", "·".dimmed(), "disabled".yellow());
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 访问日志中间件：在响应完成后输出格式化访问日志
///
/// 格式: `"<method> <uri>" <status> <latency_ms>ms`
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

/// 404 handler — returns JSON error for unknown routes.
async fn handler_404_not_found() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        axum::Json(json!({"error": "Not found"})),
    )
}
