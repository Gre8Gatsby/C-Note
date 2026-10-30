use crate::handlers::{
    calendar_summary, create_manual_note, delete_note, direct_summarize, get_note, list_contacts,
    list_notes, live_call_websocket, resummarize_note, stream_audio_file, update_note,
    upload_call_audio, AppState,
};
use axum::{
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        // Call Notes & Audio Upload
        .route("/notes/upload", post(upload_call_audio))
        .route("/notes", post(create_manual_note))
        .route("/notes", get(list_notes))
        .route("/notes/:id", get(get_note))
        .route("/notes/:id", put(update_note))
        .route("/notes/:id", delete(delete_note))
        .route("/notes/:id/resummarize", post(resummarize_note))
        .route("/notes/audio/:filename", get(stream_audio_file))
        // Direct Text Summarization
        .route("/summarize", post(direct_summarize))
        // Contacts & Calendar views
        .route("/contacts", get(list_contacts))
        .route("/calendar/summary", get(calendar_summary));

    Router::new()
        .nest("/api/v1", api_routes)
        // Live bidirectional streaming WebSocket
        .route("/ws/live-call", get(live_call_websocket))
        // Health check
        .route(
            "/health",
            get(|| async {
                Json(json!({
                    "status": "healthy",
                    "service": "c-note",
                    "version": env!("CARGO_PKG_VERSION")
                }))
            }),
        )
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

