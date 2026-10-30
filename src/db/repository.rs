use crate::error::{AppError, Result};
use crate::models::{
    CalendarDateGroup, CallNote, ContactSummary, ListNotesQuery, ListNotesResponse,
};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Clone)]
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_note(&self, note: &CallNote) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO call_notes (
                id, contact_name, phone_number, call_timestamp, duration_seconds,
                audio_file_path, transcript, summary, action_items, key_topics,
                sentiment, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&note.id)
        .bind(&note.contact_name)
        .bind(&note.phone_number)
        .bind(&note.call_timestamp)
        .bind(note.duration_seconds)
        .bind(&note.audio_file_path)
        .bind(&note.transcript)
        .bind(&note.summary)
        .bind(&note.action_items)
        .bind(&note.key_topics)
        .bind(&note.sentiment)
        .bind(&note.created_at)
        .bind(&note.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_note_by_id(&self, id: &str) -> Result<CallNote> {
        let note = sqlx::query_as::<_, CallNote>(
            "SELECT * FROM call_notes WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Call note with id '{}' not found", id)))?;

        Ok(note)
    }

    pub async fn update_note(
        &self,
        id: &str,
        contact_name: Option<String>,
        phone_number: Option<String>,
        transcript: Option<String>,
        summary: Option<String>,
        action_items: Option<String>,
    ) -> Result<CallNote> {
        let existing = self.get_note_by_id(id).await?;
        let now = Utc::now().to_rfc3339();

        let updated_contact = contact_name.unwrap_or(existing.contact_name);
        let updated_phone = phone_number.unwrap_or(existing.phone_number);
        let updated_transcript = transcript.unwrap_or(existing.transcript);
        let updated_summary = summary.unwrap_or(existing.summary);
        let updated_action_items = action_items.or(existing.action_items);

        sqlx::query(
            r#"
            UPDATE call_notes
            SET contact_name = ?, phone_number = ?, transcript = ?, summary = ?, action_items = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&updated_contact)
        .bind(&updated_phone)
        .bind(&updated_transcript)
        .bind(&updated_summary)
        .bind(&updated_action_items)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_note_by_id(id).await
    }

    pub async fn delete_note(&self, id: &str) -> Result<CallNote> {
        let note = self.get_note_by_id(id).await?;
        sqlx::query("DELETE FROM call_notes WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(note)
    }

    pub async fn list_notes(&self, query: ListNotesQuery) -> Result<ListNotesResponse> {
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let offset = query.offset.unwrap_or(0).max(0);

        let mut conditions = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(date) = query.date {
            if !date.is_empty() {
                conditions.push("call_timestamp LIKE ?");
                bindings.push(format!("{}%", date));
            }
        }

        if let Some(contact) = query.contact {
            if !contact.is_empty() {
                conditions.push("contact_name LIKE ?");
                bindings.push(format!("%{}%", contact));
            }
        }

        if let Some(phone) = query.phone {
            if !phone.is_empty() {
                conditions.push("phone_number LIKE ?");
                bindings.push(format!("%{}%", phone));
            }
        }

        if let Some(q) = query.query {
            if !q.is_empty() {
                conditions.push("(transcript LIKE ? OR summary LIKE ? OR contact_name LIKE ?)");
                let pattern = format!("%{}%", q);
                bindings.push(pattern.clone());
                bindings.push(pattern.clone());
                bindings.push(pattern);
            }
        }

        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) as count FROM call_notes {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for b in &bindings {
            count_query = count_query.bind(b);
        }
        let total_count = count_query.fetch_one(&self.pool).await?;

        let select_sql = format!(
            "SELECT * FROM call_notes {} ORDER BY call_timestamp DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut select_query = sqlx::query_as::<_, CallNote>(&select_sql);
        for b in &bindings {
            select_query = select_query.bind(b);
        }
        select_query = select_query.bind(limit).bind(offset);

        let notes = select_query.fetch_all(&self.pool).await?;

        Ok(ListNotesResponse { notes, total_count })
    }

    pub async fn get_calendar_summary(&self) -> Result<Vec<CalendarDateGroup>> {
        let rows = sqlx::query_as::<_, CalendarDateGroup>(
            r#"
            SELECT substr(call_timestamp, 1, 10) as date, count(*) as count
            FROM call_notes
            GROUP BY substr(call_timestamp, 1, 10)
            ORDER BY date DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_contacts(&self) -> Result<Vec<ContactSummary>> {
        let rows = sqlx::query_as::<_, ContactSummary>(
            r#"
            SELECT
                contact_name,
                phone_number,
                count(*) as call_count,
                max(call_timestamp) as last_call_timestamp
            FROM call_notes
            GROUP BY contact_name, phone_number
            ORDER BY last_call_timestamp DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn log_processing(
        &self,
        note_id: Option<&str>,
        service_type: &str,
        status: &str,
        duration_ms: i64,
        details: Option<&str>,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO processing_logs (id, note_id, service_type, status, duration_ms, details, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(note_id)
        .bind(service_type)
        .bind(status)
        .bind(duration_ms)
        .bind(details)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

