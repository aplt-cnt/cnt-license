use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;

use crate::models::license::ErrorResponse;
use crate::state::SharedState;

/// 获取所有许可证模板列表
/// GET / 和 GET /licenses
/// 默认返回 YAML（兼容 clicense update），Accept: application/json 时返回 JSON
pub async fn list_all(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let accept_json = wants_json(&headers);

    if accept_json {
        Json(&state.templates).into_response()
    } else {
        let yaml = serde_yaml::to_string(&state.templates).unwrap_or_default();
        (StatusCode::OK, [(header::CONTENT_TYPE, "application/yaml")], yaml).into_response()
    }
}

/// 获取单个许可证模板
/// GET /licenses/{id}
/// 默认返回 YAML，Accept: application/json 时返回 JSON
pub async fn get_one(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> Response {
    let accept_json = wants_json(&headers);

    match state.templates.get(&id) {
        Some(template) => {
            if accept_json {
                let mut map = HashMap::new();
                map.insert(&id, template.clone());
                Json(&map).into_response()
            } else {
                let mut map = HashMap::new();
                map.insert(&id, template.clone());
                let yaml = serde_yaml::to_string(&map).unwrap_or_default();
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/yaml")],
                    yaml,
                )
                    .into_response()
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("License '{}' not found", id),
                id: Some(id),
            }),
        )
            .into_response(),
    }
}

/// 获取许可证元信息
/// GET /licenses/{id}/info
/// 始终返回 JSON
pub async fn get_info(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Response {
    match state.meta.get(&id) {
        Some(meta) => Json(meta).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("License '{}' not found", id),
                id: Some(id),
            }),
        )
            .into_response(),
    }
}

/// 检查 Accept header 是否明确请求 JSON
fn wants_json(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("application/json"))
        .unwrap_or(false)
}
