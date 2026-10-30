use crate::error::{AppError, Result};
use crate::handlers::notes::AppState;
use crate::models::{DirectSummarizeRequest, DirectSummarizeResponse};
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn direct_summarize(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DirectSummarizeRequest>,
) -> Result<Json<DirectSummarizeResponse>> {
    if payload.text.trim().is_empty() {
        return Err(AppError::BadRequest("Text to summarize cannot be empty".to_string()));
    }

    let result = state
        .llm_client
        .summarize(&payload.text, payload.instructions.as_deref())
        .await?;

    Ok(Json(result))
}

