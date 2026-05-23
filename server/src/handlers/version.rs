use axum::{Json, extract::State};
use serde_json::json;

use crate::state::SharedState;

pub async fn get_version(State(state): State<SharedState>) -> Json<serde_json::Value> {
    Json(json!({ "version": state.version }))
}
