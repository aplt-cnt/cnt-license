mod config;
mod data;
mod handlers;
mod models;
mod state;

use axum::{Router, http::StatusCode, response::Json, routing::get};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("cnt_license_server=debug,tower_http=debug")
        .init();

    let config = config::Config::from_env();
    let licenses_path = config.licenses_dir.clone();
    let app_state = state::init_state(&licenses_path)
        .unwrap_or_else(|e| panic!("Failed to load license data from '{}': {}", licenses_path.display(), e));
    let shared: state::SharedState = Arc::new(app_state);

    // --- /api/v1 — primary RESTful routes ---
    let api_v1 = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/version", get(handlers::version::get_version))
        .route("/licenses", get(handlers::licenses::list_all))
        .route("/licenses/{id}", get(handlers::licenses::get_one))
        .route("/licenses/{id}/info", get(handlers::licenses::get_info))
        .route("/search", get(handlers::search::search))
        .with_state(shared.clone());

    // --- Backward-compatible aliases ---
    let compat = Router::new()
        .route("/", get(handlers::licenses::list_all))
        .route("/health", get(handlers::health::health_check))
        .route("/version", get(handlers::version::get_version))
        .route("/licenses", get(handlers::licenses::list_all))
        .route("/licenses/{id}", get(handlers::licenses::get_one))
        .route("/licenses/{id}/info", get(handlers::licenses::get_info))
        .route("/search", get(handlers::search::search))
        .with_state(shared.clone());

    // --- 404 fallback ---
    let not_found = Router::new()
        .fallback(handler_404_not_found);

    let app = Router::new()
        .merge(compat)
        .nest("/api/v1", api_v1)
        .merge(not_found)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:3000";
    tracing::info!("cnt-license-server listening on {}", addr);

    // Log the API structure
    tracing::info!("Routes:");
    tracing::info!("  GET /api/v1/health             — Health check");
    tracing::info!("  GET /api/v1/version            — Server version");
    tracing::info!("  GET /api/v1/licenses           — List all license templates");
    tracing::info!("  GET /api/v1/licenses/{{id}}     — Get one license template");
    tracing::info!("  GET /api/v1/licenses/{{id}}/info — Get license metadata");
    tracing::info!("  GET /api/v1/search?q=          — Search licenses");
    tracing::info!("  (backward-compat aliases: /health, /licenses, /search, etc.)");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// 404 handler — returns JSON error for unknown routes.
async fn handler_404_not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "Not found"})),
    )
}
