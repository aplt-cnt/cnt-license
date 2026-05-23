use axum::{Json, extract::State};
use serde_json::json;

use crate::state::SharedState;

pub async fn health_check(State(state): State<SharedState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": state.version,
    }))
}
