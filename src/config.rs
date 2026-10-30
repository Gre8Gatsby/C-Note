use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub upload_dir: String,

    // Custom Streaming STT Configuration
    pub stt_ws_url: String,
    pub stt_api_key: Option<String>,
    pub stt_sample_rate: u32,

    // Custom Streaming LLM Configuration
    pub llm_ws_url: String,
    pub llm_api_key: Option<String>,
    pub llm_model: String,

    // Fallback mode if custom streaming services are offline during dev
    pub enable_mock_fallback: bool,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://c_note.db?mode=rwc".to_string());

        let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());

        let stt_ws_url = env::var("STT_WS_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:9001/ws/stt".to_string());
        let stt_api_key = env::var("STT_API_KEY").ok();
        let stt_sample_rate = env::var("STT_SAMPLE_RATE")
            .ok()
            .and_then(|r| r.parse().ok())
            .unwrap_or(16000);

        let llm_ws_url = env::var("LLM_WS_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:9002/ws/llm".to_string());
        let llm_api_key = env::var("LLM_API_KEY").ok();
        let llm_model = env::var("LLM_MODEL")
            .unwrap_or_else(|_| "custom-call-summarizer".to_string());

        let enable_mock_fallback = env::var("ENABLE_MOCK_FALLBACK")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        Self {
            server_host,
            server_port,
            database_url,
            upload_dir,
            stt_ws_url,
            stt_api_key,
            stt_sample_rate,
            llm_ws_url,
            llm_api_key,
            llm_model,
            enable_mock_fallback,
        }
    }
}

