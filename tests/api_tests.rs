use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tower::ServiceExt;
use c_note::{
    create_router, run_migrations, AppState, AudioStorage, CallPipeline, Config, LlmClient,
    Repository, SttClient,
};

async fn setup_test_app() -> axum::Router {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.unwrap();

    let mut config = Config::from_env();
    config.enable_mock_fallback = true;
    config.upload_dir = "./target/test_uploads".to_string();

    let repo = Repository::new(pool);
    let audio_storage = AudioStorage::new(&config.upload_dir).await.unwrap();
    let stt_client = SttClient::new(config.clone());
    let llm_client = LlmClient::new(config.clone());

    let pipeline = CallPipeline::new(
        repo.clone(),
        audio_storage.clone(),
        stt_client.clone(),
        llm_client.clone(),
    );

    let state = Arc::new(AppState {
        repo,
        pipeline,
        audio_storage,
        llm_client,
        stt_client,
    });

    create_router(state)
}

#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "c-note");
}

#[tokio::test]
async fn test_direct_summarize() {
    let app = setup_test_app().await;

    let req_body = json!({
        "text": "Hi Bob, confirming our project sync tomorrow at 10 AM to review backend API documentation and deliverables."
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/summarize")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("summary").is_some());
    assert!(json.get("action_items").is_some());
    assert!(json.get("key_topics").is_some());
}

#[tokio::test]
async fn test_note_lifecycle_and_calendar() {
    let app = setup_test_app().await;

    // 1. Create a note
    let create_req = json!({
        "contact_name": "Alice Johnson",
        "phone_number": "+1234567890",
        "call_timestamp": "2026-10-30T10:00:00Z",
        "duration_seconds": 120,
        "transcript": "Alice called to discuss the product roadmap for Q4.",
        "summary": "Discussed Q4 roadmap priorities and next sprint deliverables."
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/notes")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&create_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let note: Value = serde_json::from_slice(&body).unwrap();
    let note_id = note["id"].as_str().unwrap();

    // 2. Fetch the note by ID
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/notes/{}", note_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 3. Update the note
    let update_req = json!({
        "summary": "Updated summary: sprint kickoff on Monday."
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/notes/{}", note_id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let updated_note: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        updated_note["summary"],
        "Updated summary: sprint kickoff on Monday."
    );

    // 4. Check Calendar Summary
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/calendar/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let groups: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(groups.as_array().unwrap().len(), 1);
    assert_eq!(groups[0]["date"], "2026-10-30");
    assert_eq!(groups[0]["count"], 1);

    // 5. Check Contacts List
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/contacts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let contacts: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(contacts.as_array().unwrap().len(), 1);
    assert_eq!(contacts[0]["contact_name"], "Alice Johnson");

    // 6. Delete Note
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/notes/{}", note_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_audio_call_processing_pipeline() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    run_migrations(&pool).await.unwrap();

    let mut config = Config::from_env();
    config.enable_mock_fallback = true;
    config.upload_dir = "./target/test_pipeline_uploads".to_string();

    let repo = Repository::new(pool.clone());
    let audio_storage = AudioStorage::new(&config.upload_dir).await.unwrap();
    let stt_client = SttClient::new(config.clone());
    let llm_client = LlmClient::new(config.clone());

    let pipeline = CallPipeline::new(
        repo.clone(),
        audio_storage.clone(),
        stt_client,
        llm_client,
    );

    // Simulate 16KB of audio PCM/AAC stream
    let dummy_audio = vec![0u8; 16384];

    let note = pipeline
        .process_call_audio(
            "Michael Scott".to_string(),
            "+1-800-PAPER".to_string(),
            Some("2026-10-30T14:30:00Z".to_string()),
            Some(45),
            Some("call_michael.aac"),
            &dummy_audio,
        )
        .await
        .unwrap();

    assert_eq!(note.contact_name, "Michael Scott");
    assert_eq!(note.phone_number, "+1-800-PAPER");
    assert!(!note.transcript.is_empty());
    assert!(!note.summary.is_empty());
    assert!(note.action_items.is_some());
    assert!(note.audio_file_path.is_some());

    // Verify it was saved in DB
    let fetched = repo.get_note_by_id(&note.id).await.unwrap();
    assert_eq!(fetched.id, note.id);

    // Verify audio file exists on disk
    let file_path = audio_storage.get_file_path(&note.audio_file_path.unwrap());
    assert!(file_path.exists());
}

#[tokio::test]
async fn test_live_stt_session_streaming() {
    let mut config = Config::from_env();
    config.enable_mock_fallback = true;
    let stt_client = SttClient::new(config);

    let mut session = stt_client.start_live_session().await;

    // Send 32KB of audio chunks in real-time
    session
        .send_audio_chunk(vec![0u8; 16384])
        .await
        .unwrap();
    session
        .send_audio_chunk(vec![0u8; 16384])
        .await
        .unwrap();

    // Verify events flow back
    let mut received_event = false;
    for _ in 0..5 {
        if let Some(_event) = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            session.next_event(),
        )
        .await
        .ok()
        .flatten()
        {
            received_event = true;
            break;
        }
    }
    assert!(received_event);

    let full_transcript = session.finish().await;
    assert!(!full_transcript.is_empty());
}


