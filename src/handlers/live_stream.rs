use crate::handlers::notes::AppState;
use crate::models::CallNote;
use crate::services::stt_client::LiveSttEvent;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct ClientControlMsg {
    action: Option<String>,
    contact_name: Option<String>,
    phone_number: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerLiveMsg {
    #[serde(rename = "transcript_interim")]
    Interim { text: String },
    #[serde(rename = "transcript_final")]
    Final { text: String },
    #[serde(rename = "live_action_item")]
    LiveActionItem { item: String },
    #[serde(rename = "live_topic")]
    LiveTopic { topic: String },
    #[serde(rename = "call_completed")]
    Completed { note: CallNote },
    #[serde(rename = "error")]
    Error { message: String },
}

pub async fn live_call_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_live_socket(socket, state))
}

async fn handle_live_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut client_write, mut client_read) = socket.split();
    info!("Live call real-time WebSocket client connected");

    let mut contact_name = "Live Call".to_string();
    let mut phone_number = "".to_string();
    let mut accumulated_audio = Vec::new();
    let call_start_time = Utc::now();

    // 1. Initialize real-time STT streaming session
    let mut stt_session = state.stt_client.start_live_session().await;

    // Send connection established acknowledgment
    let welcome = serde_json::json!({
        "type": "session_started",
        "message": "Real-time call transcription and live AI insights active"
    });
    let _ = client_write
        .send(Message::Text(welcome.to_string()))
        .await;

    loop {
        tokio::select! {
            // Receive events from the live STT engine
            Some(stt_event) = stt_session.next_event() => {
                match stt_event {
                    LiveSttEvent::Interim(text) => {
                        let msg = ServerLiveMsg::Interim { text };
                        if let Ok(json_str) = serde_json::to_string(&msg) {
                            let _ = client_write.send(Message::Text(json_str)).await;
                        }
                    }
                    LiveSttEvent::Final(sentence) => {
                        let msg = ServerLiveMsg::Final { text: sentence.clone() };
                        if let Ok(json_str) = serde_json::to_string(&msg) {
                            let _ = client_write.send(Message::Text(json_str)).await;
                        }

                        // Real-time Action Item & Topic detection during the call
                        if let Some(action_item) = detect_live_action_item(&sentence) {
                            let item_msg = ServerLiveMsg::LiveActionItem { item: action_item };
                            if let Ok(json_str) = serde_json::to_string(&item_msg) {
                                let _ = client_write.send(Message::Text(json_str)).await;
                            }
                        }

                        if let Some(topic) = detect_live_topic(&sentence) {
                            let topic_msg = ServerLiveMsg::LiveTopic { topic };
                            if let Ok(json_str) = serde_json::to_string(&topic_msg) {
                                let _ = client_write.send(Message::Text(json_str)).await;
                            }
                        }
                    }
                    LiveSttEvent::Error(err) => {
                        let msg = ServerLiveMsg::Error { message: err };
                        if let Ok(json_str) = serde_json::to_string(&msg) {
                            let _ = client_write.send(Message::Text(json_str)).await;
                        }
                    }
                    _ => {}
                }
            }

            // Receive audio chunks and control frames from the client
            client_msg = client_read.next() => {
                match client_msg {
                    Some(Ok(Message::Binary(bytes))) => {
                        accumulated_audio.extend_from_slice(&bytes);
                        // Forward audio frame into live STT engine in real time
                        let _ = stt_session.send_audio_chunk(bytes).await;
                    }
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(ctrl) = serde_json::from_str::<ClientControlMsg>(&text) {
                            if let Some(c) = ctrl.contact_name {
                                contact_name = c;
                            }
                            if let Some(p) = ctrl.phone_number {
                                phone_number = p;
                            }
                            if ctrl.action.as_deref() == Some("finish") {
                                info!("Received finish signal from client during live call");
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Live call client disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket read error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // 2. Conclude live STT session and get complete transcript
    let complete_transcript = stt_session.finish().await;

    // 3. Save final audio recording to disk
    let audio_filename = if !accumulated_audio.is_empty() {
        match state
            .audio_storage
            .save_audio(Some("live_call.aac"), &accumulated_audio)
            .await
        {
            Ok((name, _)) => Some(name),
            Err(e) => {
                warn!("Failed to save live call audio: {}", e);
                None
            }
        }
    } else {
        None
    };

    // 4. Run LLM Summarization on the full transcript
    let transcript = if complete_transcript.trim().is_empty() {
        "Call concluded with no speech recognized.".to_string()
    } else {
        complete_transcript
    };

    let summary_res = match state.llm_client.summarize(&transcript, None).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to summarize live call transcript: {}", e);
            crate::models::DirectSummarizeResponse {
                summary: format!("Call transcript: {}", transcript),
                action_items: vec![],
                key_topics: vec![],
                sentiment: Some("neutral".to_string()),
            }
        }
    };

    let duration_seconds = (Utc::now() - call_start_time).num_seconds().max(1);
    let now_str = Utc::now().to_rfc3339();

    let note = CallNote {
        id: Uuid::new_v4().to_string(),
        contact_name,
        phone_number,
        call_timestamp: call_start_time.to_rfc3339(),
        duration_seconds: Some(duration_seconds),
        audio_file_path: audio_filename,
        transcript,
        summary: summary_res.summary,
        action_items: serde_json::to_string(&summary_res.action_items).ok(),
        key_topics: serde_json::to_string(&summary_res.key_topics).ok(),
        sentiment: summary_res.sentiment,
        created_at: now_str.clone(),
        updated_at: now_str,
    };

    // 5. Store completed CallNote in database
    let _ = state.repo.insert_note(&note).await;
    info!("Stored live call note with id: {}", note.id);

    // 6. Send final completed note event to client
    let completed_msg = ServerLiveMsg::Completed { note };
    if let Ok(json_str) = serde_json::to_string(&completed_msg) {
        let _ = client_write.send(Message::Text(json_str)).await;
    }

    let _ = client_write.close().await;
}

/// Real-time commitment & action item detector
fn detect_live_action_item(sentence: &str) -> Option<String> {
    let lower = sentence.to_lowercase();
    let keywords = [
        "will email",
        "will send",
        "send over",
        "follow up",
        "by tomorrow",
        "by friday",
        "by monday",
        "schedule",
        "review the",
        "deliver by",
        "action item",
    ];

    for kw in &keywords {
        if lower.contains(kw) {
            return Some(sentence.trim().to_string());
        }
    }
    None
}

/// Real-time topic extractor
fn detect_live_topic(sentence: &str) -> Option<String> {
    let lower = sentence.to_lowercase();
    if lower.contains("pricing") || lower.contains("budget") || lower.contains("cost") {
        Some("Budget & Pricing".to_string())
    } else if lower.contains("timeline") || lower.contains("deadline") || lower.contains("schedule") {
        Some("Timeline & Schedule".to_string())
    } else if lower.contains("contract") || lower.contains("agreement") || lower.contains("terms") {
        Some("Contract Terms".to_string())
    } else if lower.contains("deploy") || lower.contains("architecture") || lower.contains("model") {
        Some("Technical Architecture".to_string())
    } else {
        None
    }
}
