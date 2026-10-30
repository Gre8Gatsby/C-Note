use crate::config::Config;
use crate::error::{AppError, Result};
use crate::models::DirectSummarizeResponse;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, warn};

#[derive(Debug, Serialize)]
struct LlmSummarizeRequest {
    #[serde(rename = "type")]
    msg_type: String,
    model: String,
    transcript: String,
    instructions: Option<String>,
    api_key: Option<String>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct LlmStreamResponse {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    summary: Option<String>,
    chunk: Option<String>,
    action_items: Option<Vec<String>>,
    key_topics: Option<Vec<String>>,
    sentiment: Option<String>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct LlmClient {
    config: Config,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Connects to the custom streaming LLM WebSocket to summarize a call transcript.
    pub async fn summarize(
        &self,
        transcript: &str,
        instructions: Option<&str>,
    ) -> Result<DirectSummarizeResponse> {
        let ws_url = &self.config.llm_ws_url;

        info!("Attempting to connect to custom LLM WebSocket at {}", ws_url);

        let connect_result = timeout(Duration::from_secs(4), connect_async(ws_url)).await;

        match connect_result {
            Ok(Ok((ws_stream, _response))) => {
                info!("Connected to custom LLM WebSocket server. Sending transcript...");
                self.stream_request_to_llm(ws_stream, transcript, instructions).await
            }
            Ok(Err(e)) => {
                warn!("Failed to connect to LLM WebSocket server: {}", e);
                if self.config.enable_mock_fallback {
                    info!("Mock fallback is enabled. Returning generated summary.");
                    Ok(self.generate_fallback_summary(transcript))
                } else {
                    Err(AppError::LlmService(format!("Connection error: {}", e)))
                }
            }
            Err(_) => {
                warn!("Connection to LLM WebSocket timed out after 4 seconds.");
                if self.config.enable_mock_fallback {
                    info!("Mock fallback is enabled. Returning generated summary.");
                    Ok(self.generate_fallback_summary(transcript))
                } else {
                    Err(AppError::LlmService("Connection to LLM service timed out".to_string()))
                }
            }
        }
    }

    async fn stream_request_to_llm<S>(
        &self,
        ws_stream: S,
        transcript: &str,
        instructions: Option<&str>,
    ) -> Result<DirectSummarizeResponse>
    where
        S: StreamExt<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
            + SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin,
    {
        let (mut write, mut read) = ws_stream.split();

        let req = LlmSummarizeRequest {
            msg_type: "summarize".to_string(),
            model: self.config.llm_model.clone(),
            transcript: transcript.to_string(),
            instructions: instructions.map(|s| s.to_string()),
            api_key: self.config.llm_api_key.clone(),
            stream: true,
        };

        let req_json = serde_json::to_string(&req)
            .map_err(|e| AppError::Internal(e.into()))?;

        write
            .send(Message::Text(req_json))
            .await
            .map_err(|e| AppError::LlmService(format!("Failed to send summarize request: {}", e)))?;

        let mut accumulated_text = String::new();
        let mut final_action_items: Vec<String> = Vec::new();
        let mut final_topics: Vec<String> = Vec::new();
        let mut final_sentiment: Option<String> = None;

        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(resp) = serde_json::from_str::<LlmStreamResponse>(&text) {
                        if let Some(err) = resp.error {
                            return Err(AppError::LlmService(format!("LLM server error: {}", err)));
                        }

                        if let Some(chunk) = resp.chunk {
                            accumulated_text.push_str(&chunk);
                        }

                        if let Some(summary) = resp.summary {
                            accumulated_text = summary;
                        }

                        if let Some(items) = resp.action_items {
                            final_action_items = items;
                        }

                        if let Some(topics) = resp.key_topics {
                            final_topics = topics;
                        }

                        if let Some(sent) = resp.sentiment {
                            final_sentiment = Some(sent);
                        }

                        if resp.msg_type.as_deref() == Some("done") {
                            break;
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    warn!("Error receiving stream from LLM: {}", e);
                    break;
                }
                _ => {}
            }
        }

        if accumulated_text.is_empty() {
            accumulated_text = format!("Summary of transcript: {}", transcript);
        }

        Ok(DirectSummarizeResponse {
            summary: accumulated_text,
            action_items: final_action_items,
            key_topics: final_topics,
            sentiment: final_sentiment.or_else(|| Some("neutral".to_string())),
        })
    }

    fn generate_fallback_summary(&self, transcript: &str) -> DirectSummarizeResponse {
        let first_part = transcript.chars().take(120).collect::<String>();
        DirectSummarizeResponse {
            summary: format!(
                "The conversation covered: \"{}...\". Next steps and timelines were established.",
                first_part
            ),
            action_items: vec![
                "Send updated proposal and contract details".to_string(),
                "Follow up meeting tomorrow at 2:00 PM".to_string(),
            ],
            key_topics: vec![
                "Project Timeline".to_string(),
                "Requirements & Deliverables".to_string(),
            ],
            sentiment: Some("positive".to_string()),
        }
    }
}

