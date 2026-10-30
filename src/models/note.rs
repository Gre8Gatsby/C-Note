use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CallNote {
    pub id: String,
    pub contact_name: String,
    pub phone_number: String,
    pub call_timestamp: String, // ISO-8601 string
    pub duration_seconds: Option<i64>,
    pub audio_file_path: Option<String>,
    pub transcript: String,
    pub summary: String,
    pub action_items: Option<String>, // JSON-encoded string or bullet list
    pub key_topics: Option<String>,   // JSON-encoded array or comma separated
    pub sentiment: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateManualNoteRequest {
    pub contact_name: String,
    pub phone_number: Option<String>,
    pub call_timestamp: Option<String>,
    pub duration_seconds: Option<i64>,
    pub transcript: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub contact_name: Option<String>,
    pub phone_number: Option<String>,
    pub transcript: Option<String>,
    pub summary: Option<String>,
    pub action_items: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListNotesQuery {
    pub date: Option<String>,      // YYYY-MM-DD
    pub contact: Option<String>,   // Search contact name
    pub phone: Option<String>,     // Search phone number
    pub query: Option<String>,     // Full text search across transcript & summary
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListNotesResponse {
    pub notes: Vec<CallNote>,
    pub total_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct DirectSummarizeRequest {
    pub text: String,
    pub instructions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectSummarizeResponse {
    pub summary: String,
    pub action_items: Vec<String>,
    pub key_topics: Vec<String>,
    pub sentiment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CalendarDateGroup {
    pub date: String, // YYYY-MM-DD
    pub count: i64,
}
