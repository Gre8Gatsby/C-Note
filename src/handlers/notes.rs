use crate::db::Repository;
use crate::error::{AppError, Result};
use crate::models::{
    CallNote, CreateManualNoteRequest, ListNotesQuery, ListNotesResponse, UpdateNoteRequest,
};
use crate::services::{AudioStorage, CallPipeline, LlmClient};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::header,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub repo: Repository,
    pub pipeline: CallPipeline,
    pub audio_storage: AudioStorage,
    pub llm_client: LlmClient,
}

/// Handler for multipart audio call upload
pub async fn upload_call_audio(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<CallNote>> {
    let mut contact_name = "Unknown".to_string();
    let mut phone_number = "".to_string();
    let mut call_timestamp: Option<String> = None;
    let mut duration_seconds: Option<i64> = None;
    let mut original_filename: Option<String> = None;
    let mut audio_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart parsing error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "contact_name" => {
                let text = field.text().await.unwrap_or_default();
                if !text.trim().is_empty() {
                    contact_name = text;
                }
            }
            "phone_number" => {
                phone_number = field.text().await.unwrap_or_default();
            }
            "call_timestamp" => {
                let text = field.text().await.unwrap_or_default();
                if !text.trim().is_empty() {
                    call_timestamp = Some(text);
                }
            }
            "duration_seconds" => {
                if let Ok(text) = field.text().await {
                    duration_seconds = text.parse().ok();
                }
            }
            "audio" | "file" | "recording" => {
                original_filename = field.file_name().map(|s| s.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Audio read error: {}", e)))?;
                audio_bytes = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    let bytes = audio_bytes.ok_or_else(|| {
        AppError::BadRequest("Missing audio file field ('audio', 'file', or 'recording')".to_string())
    })?;

    if bytes.is_empty() {
        return Err(AppError::BadRequest("Uploaded audio file is empty".to_string()));
    }

    let note = state
        .pipeline
        .process_call_audio(
            contact_name,
            phone_number,
            call_timestamp,
            duration_seconds,
            original_filename.as_deref(),
            &bytes,
        )
        .await?;

    Ok(Json(note))
}

/// Create a manual note (no audio recording)
pub async fn create_manual_note(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateManualNoteRequest>,
) -> Result<Json<CallNote>> {
    let now_str = Utc::now().to_rfc3339();
    let note_id = Uuid::new_v4().to_string();
    let transcript = payload.transcript.unwrap_or_default();

    let (summary, action_items, key_topics, sentiment) = if let Some(s) = payload.summary {
        (s, None, None, None)
    } else if !transcript.is_empty() {
        let resp = state.llm_client.summarize(&transcript, None).await?;
        (
            resp.summary,
            serde_json::to_string(&resp.action_items).ok(),
            serde_json::to_string(&resp.key_topics).ok(),
            resp.sentiment,
        )
    } else {
        ("Manual note created without transcript".to_string(), None, None, None)
    };

    let note = CallNote {
        id: note_id,
        contact_name: payload.contact_name,
        phone_number: payload.phone_number.unwrap_or_default(),
        call_timestamp: payload.call_timestamp.unwrap_or_else(|| now_str.clone()),
        duration_seconds: payload.duration_seconds,
        audio_file_path: None,
        transcript,
        summary,
        action_items,
        key_topics,
        sentiment,
        created_at: now_str.clone(),
        updated_at: now_str,
    };

    state.repo.insert_note(&note).await?;

    Ok(Json(note))
}

/// List notes with search, contact, phone, or date filtering
pub async fn list_notes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListNotesQuery>,
) -> Result<Json<ListNotesResponse>> {
    let res = state.repo.list_notes(query).await?;
    Ok(Json(res))
}

/// Get single note by ID
pub async fn get_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<CallNote>> {
    let note = state.repo.get_note_by_id(&id).await?;
    Ok(Json(note))
}

/// Update note fields
pub async fn update_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<CallNote>> {
    let updated = state
        .repo
        .update_note(
            &id,
            payload.contact_name,
            payload.phone_number,
            payload.transcript,
            payload.summary,
            payload.action_items,
        )
        .await?;
    Ok(Json(updated))
}

/// Delete note by ID
pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let deleted = state.repo.delete_note(&id).await?;

    if let Some(audio_file) = deleted.audio_file_path {
        let _ = state.audio_storage.delete_audio(&audio_file).await;
    }

    Ok(Json(json!({
        "status": "success",
        "deleted_id": deleted.id
    })))
}

/// Re-summarize an existing note using the LLM client
pub async fn resummarize_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<CallNote>> {
    let note = state.repo.get_note_by_id(&id).await?;

    if note.transcript.is_empty() {
        return Err(AppError::BadRequest("Note has no transcript to summarize".to_string()));
    }

    let summary_res = state.llm_client.summarize(&note.transcript, None).await?;
    let action_items_json = serde_json::to_string(&summary_res.action_items).ok();

    let updated = state
        .repo
        .update_note(
            &id,
            None,
            None,
            None,
            Some(summary_res.summary),
            action_items_json,
        )
        .await?;

    Ok(Json(updated))
}

/// Stream the stored audio recording file
pub async fn stream_audio_file(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse> {
    // Sanitize filename to prevent directory traversal
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(AppError::BadRequest("Invalid audio filename".to_string()));
    }

    let file_path = state.audio_storage.get_file_path(&filename);
    if !file_path.exists() {
        return Err(AppError::NotFound("Audio recording file not found".to_string()));
    }

    let file = File::open(&file_path).await?;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let mime_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    let headers = [
        (header::CONTENT_TYPE, mime_type),
        (
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", filename),
        ),
    ];

    Ok((headers, body))
}
