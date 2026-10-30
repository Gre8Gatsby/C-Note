use crate::db::Repository;
use crate::error::Result;
use crate::models::CallNote;
use crate::services::audio_storage::AudioStorage;
use crate::services::llm_client::LlmClient;
use crate::services::stt_client::SttClient;
use chrono::Utc;
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct CallPipeline {
    repo: Repository,
    audio_storage: AudioStorage,
    stt_client: SttClient,
    llm_client: LlmClient,
}

impl CallPipeline {
    pub fn new(
        repo: Repository,
        audio_storage: AudioStorage,
        stt_client: SttClient,
        llm_client: LlmClient,
    ) -> Self {
        Self {
            repo,
            audio_storage,
            stt_client,
            llm_client,
        }
    }

    /// Process a recorded audio call: save audio, run STT stream, run LLM summarization, save note.
    pub async fn process_call_audio(
        &self,
        contact_name: String,
        phone_number: String,
        call_timestamp: Option<String>,
        duration_seconds: Option<i64>,
        original_filename: Option<&str>,
        audio_bytes: &[u8],
    ) -> Result<CallNote> {
        let note_id = Uuid::new_v4().to_string();
        let now_str = Utc::now().to_rfc3339();
        let timestamp = call_timestamp.unwrap_or_else(|| now_str.clone());

        // 1. Save audio file locally
        let (audio_filename, _path) = self
            .audio_storage
            .save_audio(original_filename, audio_bytes)
            .await?;
        info!("Audio saved as: {}", audio_filename);

        // 2. Transcribe via Streaming STT
        let stt_start = Instant::now();
        let transcript = match self.stt_client.transcribe_audio_stream(audio_bytes).await {
            Ok(t) => {
                let duration_ms = stt_start.elapsed().as_millis() as i64;
                let _ = self
                    .repo
                    .log_processing(Some(&note_id), "STT", "SUCCESS", duration_ms, None)
                    .await;
                t
            }
            Err(e) => {
                let duration_ms = stt_start.elapsed().as_millis() as i64;
                let _ = self
                    .repo
                    .log_processing(
                        Some(&note_id),
                        "STT",
                        "FAILED",
                        duration_ms,
                        Some(&e.to_string()),
                    )
                    .await;
                return Err(e);
            }
        };

        // 3. Summarize via Streaming LLM
        let llm_start = Instant::now();
        let summary_res = match self.llm_client.summarize(&transcript, None).await {
            Ok(s) => {
                let duration_ms = llm_start.elapsed().as_millis() as i64;
                let _ = self
                    .repo
                    .log_processing(Some(&note_id), "LLM", "SUCCESS", duration_ms, None)
                    .await;
                s
            }
            Err(e) => {
                let duration_ms = llm_start.elapsed().as_millis() as i64;
                let _ = self
                    .repo
                    .log_processing(
                        Some(&note_id),
                        "LLM",
                        "FAILED",
                        duration_ms,
                        Some(&e.to_string()),
                    )
                    .await;
                return Err(e);
            }
        };

        let action_items_json = serde_json::to_string(&summary_res.action_items).ok();
        let key_topics_json = serde_json::to_string(&summary_res.key_topics).ok();

        // 4. Construct CallNote
        let note = CallNote {
            id: note_id,
            contact_name,
            phone_number,
            call_timestamp: timestamp,
            duration_seconds,
            audio_file_path: Some(audio_filename),
            transcript,
            summary: summary_res.summary,
            action_items: action_items_json,
            key_topics: key_topics_json,
            sentiment: summary_res.sentiment,
            created_at: now_str.clone(),
            updated_at: now_str,
        };

        // 5. Save to database
        self.repo.insert_note(&note).await?;
        info!("Successfully processed and stored CallNote id: {}", note.id);

        Ok(note)
    }
}

