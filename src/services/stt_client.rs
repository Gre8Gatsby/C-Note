use crate::config::Config;
use crate::error::{AppError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
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

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "text")]
pub enum LiveSttEvent {
    #[serde(rename = "transcript_interim")]
    Interim(String),
    #[serde(rename = "transcript_final")]
    Final(String),
    #[serde(rename = "transcript_complete")]
    Complete(String),
    #[serde(rename = "transcript_error")]
    Error(String),
}

pub struct LiveSttSession {
    audio_tx: mpsc::Sender<Vec<u8>>,
    event_rx: mpsc::Receiver<LiveSttEvent>,
    finish_tx: Option<oneshot::Sender<()>>,
    result_rx: Option<oneshot::Receiver<String>>,
}

impl LiveSttSession {
    pub async fn send_audio_chunk(&self, chunk: Vec<u8>) -> Result<()> {
        self.audio_tx
            .send(chunk)
            .await
            .map_err(|_| AppError::SttService("STT worker closed unexpectedly".to_string()))
    }

    pub async fn next_event(&mut self) -> Option<LiveSttEvent> {
        self.event_rx.recv().await
    }

    pub async fn finish(&mut self) -> String {
        if let Some(tx) = self.finish_tx.take() {
            let _ = tx.send(());
        }
        if let Some(rx) = self.result_rx.take() {
            rx.await.unwrap_or_else(|_| "".to_string())
        } else {
            "".to_string()
        }
    }
}

#[derive(Clone)]
pub struct SttClient {
    config: Config,
}

impl SttClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Starts a live, streaming Speech-To-Text session over WebSocket.
    pub async fn start_live_session(&self) -> LiveSttSession {
        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(100);
        let (event_tx, event_rx) = mpsc::channel::<LiveSttEvent>(100);
        let (finish_tx, mut finish_rx) = oneshot::channel::<()>();
        let (result_tx, result_rx) = oneshot::channel::<String>();

        let ws_url = self.config.stt_ws_url.clone();
        let sample_rate = self.config.stt_sample_rate;
        let api_key = self.config.stt_api_key.clone();
        let enable_mock = self.config.enable_mock_fallback;

        tokio::spawn(async move {
            info!("Live STT: Connecting to {}", ws_url);
            let connect_res = timeout(Duration::from_secs(3), connect_async(&ws_url)).await;

            match connect_res {
                Ok(Ok((ws_stream, _))) => {
                    info!("Live STT: Connected to real-time STT WebSocket");
                    let (mut write, mut read) = ws_stream.split();

                    let start_msg = SttStartConfig {
                        msg_type: "start".to_string(),
                        sample_rate,
                        encoding: "pcm_s16le".to_string(),
                        language: "en".to_string(),
                        api_key,
                    };

                    if let Ok(json_str) = serde_json::to_string(&start_msg) {
                        let _ = write.send(Message::Text(json_str)).await;
                    }

                    let mut full_transcript = Vec::new();

                    loop {
                        tokio::select! {
                            biased;
                            _ = &mut finish_rx => {
                                let stop_msg = serde_json::json!({ "type": "stop" }).to_string();
                                let _ = write.send(Message::Text(stop_msg)).await;
                                break;
                            }
                            Some(chunk) = audio_rx.recv() => {
                                if let Err(e) = write.send(Message::Binary(chunk)).await {
                                    warn!("Live STT write error: {}", e);
                                    break;
                                }
                            }
                            Some(msg) = read.next() => {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        if let Ok(resp) = serde_json::from_str::<SttServerResponse>(&text) {
                                            if let Some(t) = resp.transcript.or(resp.text) {
                                                if resp.msg_type.as_deref() == Some("interim") {
                                                    let _ = event_tx.send(LiveSttEvent::Interim(t)).await;
                                                } else {
                                                    full_transcript.push(t.clone());
                                                    let _ = event_tx.send(LiveSttEvent::Final(t)).await;
                                                }
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => break,
                                    Err(_) => break,
                                    _ => {}
                                }
                            }
                            else => break,
                        }
                    }

                    let consolidated = full_transcript.join(" ");
                    let _ = result_tx.send(consolidated);
                }
                _ => {
                    if enable_mock {
                        info!("Live STT: External STT server offline; running live simulation worker");
                        run_mock_live_stt(audio_rx, event_tx, finish_rx, result_tx).await;
                    } else {
                        let _ = event_tx
                            .send(LiveSttEvent::Error("STT service unreachable".to_string()))
                            .await;
                        let _ = result_tx.send("".to_string());
                    }
                }
            }
        });

        LiveSttSession {
            audio_tx,
            event_rx,
            finish_tx: Some(finish_tx),
            result_rx: Some(result_rx),
        }
    }

    /// Streams an audio buffer in chunks to the custom WebSocket STT server.
    pub async fn transcribe_audio_stream(&self, audio_bytes: &[u8]) -> Result<String> {
        let ws_url = &self.config.stt_ws_url;
        let connect_result = timeout(Duration::from_secs(4), connect_async(ws_url)).await;

        match connect_result {
            Ok(Ok((ws_stream, _response))) => {
                self.stream_chunks_to_stt(ws_stream, audio_bytes).await
            }
            Ok(Err(e)) => {
                if self.config.enable_mock_fallback {
                    Ok(self.generate_fallback_transcript(audio_bytes))
                } else {
                    Err(AppError::SttService(format!("Connection error: {}", e)))
                }
            }
            Err(_) => {
                if self.config.enable_mock_fallback {
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

        const CHUNK_SIZE: usize = 8192;
        for chunk in audio_bytes.chunks(CHUNK_SIZE) {
            write
                .send(Message::Binary(chunk.to_vec()))
                .await
                .map_err(|e| AppError::SttService(format!("Failed sending audio chunk: {}", e)))?;
        }

        let stop_msg = serde_json::json!({ "type": "stop" }).to_string();
        write
            .send(Message::Text(stop_msg))
            .await
            .map_err(|e| AppError::SttService(format!("Failed to send stop frame: {}", e)))?;

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

/// Realistic live speech recognition simulation when in local testing mode
async fn run_mock_live_stt(
    mut audio_rx: mpsc::Receiver<Vec<u8>>,
    event_tx: mpsc::Sender<LiveSttEvent>,
    mut finish_rx: oneshot::Receiver<()>,
    result_tx: oneshot::Sender<String>,
) {
    let mock_sentences = [
        "Hello, thanks for taking the time to speak today.",
        "We are looking into deploying the new C Note real-time speech model next month.",
        "Could you send over the technical specification and the latency guarantees?",
        "Yes, absolutely. I will email the SLA benchmarks and revised agreement by tomorrow at 2 PM.",
        "Perfect, let's schedule our engineering sync for Friday.",
    ];

    let mut sentence_idx = 0;
    let mut total_bytes = 0usize;
    let mut byte_threshold = 16384;
    let mut full_transcript = Vec::new();

    loop {
        tokio::select! {
            biased;
            _ = &mut finish_rx => {
                break;
            }
            Some(chunk) = audio_rx.recv() => {
                total_bytes += chunk.len();

                if sentence_idx < mock_sentences.len() && total_bytes >= byte_threshold {
                    let sentence = mock_sentences[sentence_idx];
                    let words: Vec<&str> = sentence.split_whitespace().collect();

                    // Send interim progressive tokens
                    for step in 1..=words.len() {
                        let partial = words[0..step].join(" ");
                        let _ = event_tx.send(LiveSttEvent::Interim(partial)).await;
                        tokio::time::sleep(Duration::from_millis(60)).await;
                    }

                    // Emit finalized sentence
                    let _ = event_tx.send(LiveSttEvent::Final(sentence.to_string())).await;
                    full_transcript.push(sentence.to_string());

                    sentence_idx += 1;
                    byte_threshold += 24000;
                }
            }
            else => break,
        }
    }

    let final_text = if full_transcript.is_empty() {
        "Call conversation recorded and transcribed successfully.".to_string()
    } else {
        full_transcript.join(" ")
    };

    let _ = event_tx
        .send(LiveSttEvent::Complete(final_text.clone()))
        .await;
    let _ = result_tx.send(final_text);
}
