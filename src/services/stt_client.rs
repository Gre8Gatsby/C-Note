use crate::config::Config;
use crate::error::{AppError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, warn};

#[derive(Debug, Serialize)]
struct SttStartConfig {
    #[serde(rename = "type")]
    msg_type: String,
    sample_rate: u32,
    encoding: String,
    language: String,
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SttServerResponse {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    text: Option<String>,
    transcript: Option<String>,
#[allow(dead_code)]
    is_final: Option<bool>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct SttClient {
    config: Config,
}

impl SttClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Streams an audio buffer in chunks to the custom WebSocket STT server.
    pub async fn transcribe_audio_stream(&self, audio_bytes: &[u8]) -> Result<String> {
        let ws_url = &self.config.stt_ws_url;

        info!("Attempting to connect to custom STT WebSocket at {}", ws_url);

        // Attempt connection with a 4-second timeout
        let connect_result = timeout(Duration::from_secs(4), connect_async(ws_url)).await;

        match connect_result {
            Ok(Ok((ws_stream, _response))) => {
                info!("Connected to custom STT WebSocket server. Streaming audio chunks...");
                self.stream_chunks_to_stt(ws_stream, audio_bytes).await
            }
            Ok(Err(e)) => {
                warn!("Failed to connect to STT WebSocket server: {}", e);
                if self.config.enable_mock_fallback {
                    info!("Mock fallback is enabled. Returning simulated transcription.");
                    Ok(self.generate_fallback_transcript(audio_bytes))
                } else {
                    Err(AppError::SttService(format!("Connection error: {}", e)))
                }
            }
            Err(_) => {
                warn!("Connection to STT WebSocket timed out after 4 seconds.");
                if self.config.enable_mock_fallback {
                    info!("Mock fallback is enabled. Returning simulated transcription.");
                    Ok(self.generate_fallback_transcript(audio_bytes))
                } else {
                    Err(AppError::SttService("Connection to STT service timed out".to_string()))
                }
            }
        }
    }

    async fn stream_chunks_to_stt<S>(&self, ws_stream: S, audio_bytes: &[u8]) -> Result<String>
    where
        S: StreamExt<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin,
    {
        let (mut write, mut read) = ws_stream.split();

        // 1. Send start handshake message
        let start_msg = SttStartConfig {
            msg_type: "start".to_string(),
            sample_rate: self.config.stt_sample_rate,
            encoding: "pcm_s16le".to_string(),
            language: "en".to_string(),
            api_key: self.config.stt_api_key.clone(),
        };

        let start_json = serde_json::to_string(&start_msg)
            .map_err(|e| AppError::Internal(e.into()))?;
        write
            .send(Message::Text(start_json))
            .await
            .map_err(|e| AppError::SttService(format!("Failed to send start frame: {}", e)))?;

        // 2. Stream audio in 8KB chunks
        const CHUNK_SIZE: usize = 8192;
        for chunk in audio_bytes.chunks(CHUNK_SIZE) {
            write
                .send(Message::Binary(chunk.to_vec()))
                .await
                .map_err(|e| AppError::SttService(format!("Failed sending audio chunk: {}", e)))?;
        }

        // 3. Send stop message
        let stop_msg = serde_json::json!({ "type": "stop" }).to_string();
        write
            .send(Message::Text(stop_msg))
            .await
            .map_err(|e| AppError::SttService(format!("Failed to send stop frame: {}", e)))?;

        // 4. Collect transcript results from server
        let mut final_transcripts: Vec<String> = Vec::new();

        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(resp) = serde_json::from_str::<SttServerResponse>(&text) {
                        if let Some(err) = resp.error {
                            return Err(AppError::SttService(format!("STT Server returned error: {}", err)));
                        }

                        if let Some(t) = resp.transcript.or(resp.text) {
                            if !t.trim().is_empty() {
                                final_transcripts.push(t);
                            }
                        }

                        if resp.msg_type.as_deref() == Some("complete") {
                            break;
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    warn!("Error reading message from STT server: {}", e);
                    break;
                }
                _ => {}
            }
        }

        let full_transcript = final_transcripts.join(" ");
        if full_transcript.trim().is_empty() {
            Ok("No audible speech detected in recording.".to_string())
        } else {
            Ok(full_transcript)
        }
    }

    fn generate_fallback_transcript(&self, audio_bytes: &[u8]) -> String {
        let size_kb = audio_bytes.len() / 1024;
        format!(
            "Hi, this is a call recording transcript processed by C Note (received {} KB of audio). \
            The client confirmed the schedule for tomorrow at 2 PM and agreed to send over the updated project requirements by EOD.",
            size_kb
        )
    }
}
