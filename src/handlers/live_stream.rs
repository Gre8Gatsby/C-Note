use crate::handlers::notes::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct ClientControlMsg {
    action: Option<String>,
    contact_name: Option<String>,
    phone_number: Option<String>,
}

#[derive(Debug, Serialize)]
struct ServerLiveEvent {
    #[serde(rename = "type")]
    event_type: String,
    text: String,
    is_final: bool,
}

pub async fn live_call_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_live_socket(socket, state))
}

async fn handle_live_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut client_write, mut client_read) = socket.split();
    info!("Live call WebSocket client connected");

    let mut contact_name = "Live Call".to_string();
    let mut phone_number = "".to_string();
    let mut accumulated_audio = Vec::new();

    while let Some(msg_result) = client_read.next().await {
        match msg_result {
            Ok(Message::Binary(bytes)) => {
                accumulated_audio.extend_from_slice(&bytes);

                // Send live acknowledgment / progress back to client
                let live_event = ServerLiveEvent {
                    event_type: "audio_chunk_received".to_string(),
                    text: format!("Buffered {} KB", accumulated_audio.len() / 1024),
                    is_final: false,
                };
                if let Ok(json_str) = serde_json::to_string(&live_event) {
                    let _ = client_write.send(Message::Text(json_str)).await;
                }
            }
            Ok(Message::Text(text)) => {
                if let Ok(ctrl) = serde_json::from_str::<ClientControlMsg>(&text) {
                    if let Some(c) = ctrl.contact_name {
                        contact_name = c;
                    }
                    if let Some(p) = ctrl.phone_number {
                        phone_number = p;
                    }
                    if ctrl.action.as_deref() == Some("finish") {
                        info!("Received finish signal from client. Running processing pipeline...");
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Live call client disconnected");
                break;
            }
            Err(e) => {
                warn!("WebSocket read error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // If audio was collected, process the finished call
    if !accumulated_audio.is_empty() {
        info!("Processing {} bytes of live call audio...", accumulated_audio.len());
        match state
            .pipeline
            .process_call_audio(
                contact_name,
                phone_number,
                None,
                None,
                Some("live_recording.aac"),
                &accumulated_audio,
            )
            .await
        {
            Ok(note) => {
                let note_json = serde_json::to_string(&serde_json::json!({
                    "type": "call_completed",
                    "note": note
                }))
                .unwrap_or_default();
                let _ = client_write.send(Message::Text(note_json)).await;
            }
            Err(e) => {
                let err_json = serde_json::to_string(&serde_json::json!({
                    "type": "error",
                    "message": e.to_string()
                }))
                .unwrap_or_default();
                let _ = client_write.send(Message::Text(err_json)).await;
            }
        }
    }

    let _ = client_write.close().await;
}
