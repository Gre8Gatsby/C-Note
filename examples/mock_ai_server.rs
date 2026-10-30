use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SttStartMsg {
    #[serde(rename = "type")]
    msg_type: String,
    sample_rate: Option<u32>,
    encoding: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Serialize)]
struct SttEvent {
    #[serde(rename = "type")]
    msg_type: String,
    transcript: String,
    is_final: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LlmReqMsg {
    #[serde(rename = "type")]
    msg_type: String,
    transcript: String,
    instructions: Option<String>,
}

#[derive(Debug, Serialize)]
struct LlmStreamEvent {
    #[serde(rename = "type")]
    msg_type: String,
    summary: Option<String>,
    chunk: Option<String>,
    action_items: Option<Vec<String>>,
    key_topics: Option<Vec<String>>,
    sentiment: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("============================================================");
    println!(" Verbalyze Mock Streaming AI Services");
    println!(" - Mock STT WebSocket: ws://127.0.0.1:9001/ws/stt");
    println!(" - Mock LLM WebSocket: ws://127.0.0.1:9002/ws/llm");
    println!("============================================================");

    // Spawn STT Server on port 9001
    tokio::spawn(async {
        let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
        println!(" [STT] Listening on ws://127.0.0.1:9001");
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_stt_connection(stream));
        }
    });

    // Spawn LLM Server on port 9002
    let listener = TcpListener::bind("127.0.0.1:9002").await?;
    println!(" [LLM] Listening on ws://127.0.0.1:9002");
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_llm_connection(stream));
    }

    Ok(())
}

async fn handle_stt_connection(stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(stream).await.unwrap();
    println!(" [STT] Client connected");
    let (mut write, mut read) = ws_stream.split();

    let mut byte_count = 0;
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(text) => {
                println!(" [STT] Received control: {}", text);
                if text.contains("\"stop\"") {
                    println!(" [STT] Stream stopped. Sending final transcription...");
                    let resp = SttEvent {
                        msg_type: "complete".to_string(),
                        transcript: format!(
                            "Client called to finalize the contract terms and requested an updated estimate for next quarter. Received {} bytes of audio stream.",
                            byte_count
                        ),
                        is_final: true,
                    };
                    let json = serde_json::to_string(&resp).unwrap();
                    let _ = write.send(Message::Text(json)).await;
                    break;
                }
            }
            Message::Binary(bin) => {
                byte_count += bin.len();
                // Send interim hypothesis
                let interim = SttEvent {
                    msg_type: "interim".to_string(),
                    transcript: format!("Transcribing... (buffered {} KB)", byte_count / 1024),
                    is_final: false,
                };
                let _ = write
                    .send(Message::Text(serde_json::to_string(&interim).unwrap()))
                    .await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    let _ = write.close().await;
}

async fn handle_llm_connection(stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(stream).await.unwrap();
    println!(" [LLM] Client connected");
    let (mut write, mut read) = ws_stream.split();

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Text(text) = msg {
            println!(" [LLM] Summarize request: {}", text);
            if let Ok(req) = serde_json::from_str::<LlmReqMsg>(&text) {
                // Stream chunks
                let chunks = vec![
                    "The caller discussed ",
                    "contract finalization ",
                    "and requested an updated Q1 roadmap.",
                ];

                for chunk in chunks {
                    let event = LlmStreamEvent {
                        msg_type: "chunk".to_string(),
                        summary: None,
                        chunk: Some(chunk.to_string()),
                        action_items: None,
                        key_topics: None,
                        sentiment: None,
                    };
                    let _ = write
                        .send(Message::Text(serde_json::to_string(&event).unwrap()))
                        .await;
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }

                // Send final structured completion
                let final_event = LlmStreamEvent {
                    msg_type: "done".to_string(),
                    summary: Some(format!(
                        "Client called regarding: \"{}\". They reviewed current progress and agreed to move forward with the next milestone.",
                        req.transcript.chars().take(80).collect::<String>()
                    )),
                    chunk: None,
                    action_items: Some(vec![
                        "Email updated milestone plan to client".to_string(),
                        "Schedule technical follow-up call on Friday".to_string(),
                    ]),
                    key_topics: Some(vec![
                        "Contract Terms".to_string(),
                        "Q1 Roadmap".to_string(),
                        "Milestone Delivery".to_string(),
                    ]),
                    sentiment: Some("positive".to_string()),
                };

                let _ = write
                    .send(Message::Text(serde_json::to_string(&final_event).unwrap()))
                    .await;
                break;
            }
        }
    }
    let _ = write.close().await;
}
