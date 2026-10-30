use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use c_note::{
    create_router, run_migrations, AppState, AudioStorage, CallPipeline, Config, LlmClient,
    Repository, SttClient,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "c_note=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    info!("Starting C Note Backend Service...");
    info!("Database: {}", config.database_url);
    info!("STT WebSocket URL: {}", config.stt_ws_url);
    info!("LLM WebSocket URL: {}", config.llm_ws_url);
    info!("Mock Fallback: {}", config.enable_mock_fallback);

    // Setup SQLite connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    // Run schema migrations
    run_migrations(&pool).await?;
    info!("Database schema initialized.");

    // Initialize services
    let repo = Repository::new(pool);
    let audio_storage = AudioStorage::new(&config.upload_dir).await?;
    let stt_client = SttClient::new(config.clone());
    let llm_client = LlmClient::new(config.clone());

    let pipeline = CallPipeline::new(
        repo.clone(),
        audio_storage.clone(),
        stt_client.clone(),
        llm_client.clone(),
    );

    let state = Arc::new(AppState {
        repo,
        pipeline,
        audio_storage,
        llm_client,
        stt_client,
    });

    let app = create_router(state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🚀 C Note Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
