use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContactSummary {
    pub contact_name: String,
    pub phone_number: String,
    pub call_count: i64,
    pub last_call_timestamp: Option<String>,
}

