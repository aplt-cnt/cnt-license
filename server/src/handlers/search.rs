use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::models::license::SearchResponse;
use crate::state::SharedState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub async fn search(
    State(state): State<SharedState>,
    Query(query): Query<SearchQuery>,
) -> Json<SearchResponse> {
    let q = query.q.unwrap_or_default().to_lowercase();

    let results = if q.is_empty() {
        state
            .meta
            .iter()
            .map(|(id, meta)| crate::models::license::LicenseSearchEntry {
                id: id.clone(),
                name: meta.name.clone(),
                description: meta.description.clone(),
            })
            .collect()
    } else {
        state
            .meta
            .iter()
            .filter(|(id, meta)| {
                id.to_lowercase().contains(&q)
                    || meta.name.to_lowercase().contains(&q)
                    || meta.description.to_lowercase().contains(&q)
                    || meta.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .map(|(id, meta)| crate::models::license::LicenseSearchEntry {
                id: id.clone(),
                name: meta.name.clone(),
                description: meta.description.clone(),
            })
            .collect()
    };

    Json(SearchResponse {
        query: q,
        results,
    })
}
