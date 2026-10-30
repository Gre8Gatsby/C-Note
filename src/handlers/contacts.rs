use crate::error::Result;
use crate::handlers::notes::AppState;
use crate::models::{CalendarDateGroup, ContactSummary};
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn list_contacts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ContactSummary>>> {
    let contacts = state.repo.list_contacts().await?;
    Ok(Json(contacts))
}

pub async fn calendar_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CalendarDateGroup>>> {
    let groups = state.repo.get_calendar_summary().await?;
    Ok(Json(groups))
}

