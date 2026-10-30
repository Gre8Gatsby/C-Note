use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Enable WAL mode for high concurrency
    sqlx::query("PRAGMA journal_mode = WAL;").execute(pool).await?;
    sqlx::query("PRAGMA synchronous = NORMAL;").execute(pool).await?;

    // Create call_notes table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS call_notes (
            id TEXT PRIMARY KEY NOT NULL,
            contact_name TEXT NOT NULL,
            phone_number TEXT NOT NULL,
            call_timestamp TEXT NOT NULL,
            duration_seconds INTEGER,
            audio_file_path TEXT,
            transcript TEXT NOT NULL,
            summary TEXT NOT NULL,
            action_items TEXT,
            key_topics TEXT,
            sentiment TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_notes_call_timestamp ON call_notes (call_timestamp);
        CREATE INDEX IF NOT EXISTS idx_notes_contact_name ON call_notes (contact_name);
        CREATE INDEX IF NOT EXISTS idx_notes_phone_number ON call_notes (phone_number);

        -- Table for tracking STT and LLM pipeline execution logs & performance
        CREATE TABLE IF NOT EXISTS processing_logs (
            id TEXT PRIMARY KEY NOT NULL,
            note_id TEXT,
            service_type TEXT NOT NULL, -- 'STT' or 'LLM'
            status TEXT NOT NULL,       -- 'SUCCESS', 'FAILED', 'FALLBACK'
            duration_ms INTEGER NOT NULL,
            details TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (note_id) REFERENCES call_notes (id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

