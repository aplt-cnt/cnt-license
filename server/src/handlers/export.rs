use axum::{
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};

use crate::state::SharedState;

/// GET /api/v1/export — exports all license templates and metadata as a .zip file.
pub async fn export(State(state): State<SharedState>) -> impl IntoResponse {
    match crate::data::build_zip(&state.templates, &state.meta) {
        Ok(zip_bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/zip"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"clicense-export.zip\"",
                ),
            ],
            zip_bytes,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build export: {}", e),
        )
            .into_response(),
    }
}
