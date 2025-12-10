use futures::stream::{Stream, StreamExt};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};
use systemprompt_core_logging::LogService;
use tokio::sync::mpsc;

use super::error::{ClientError, ClientResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    Content(String),
    Tool(Value),
    Complete,
    Error(String),
}

#[derive(Debug)]
pub struct SseStream {
    receiver: mpsc::Receiver<ClientResult<StreamEvent>>,
}

impl SseStream {
    pub async fn new(client: Client, url: String, body: Value) -> ClientResult<Self> {
        Self::new_with_logger(client, url, body, None).await
    }

    pub async fn new_with_logger(
        client: Client,
        url: String,
        body: Value,
        log_service: Option<LogService>,
    ) -> ClientResult<Self> {
        Self::new_with_auth(client, url, body, None, log_service).await
    }

    pub async fn new_with_auth(
        client: Client,
        url: String,
        body: Value,
        auth_token: Option<String>,
        log_service: Option<LogService>,
    ) -> ClientResult<Self> {
        let (sender, receiver) = mpsc::channel(100);

        let mut request = client
            .post(&url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .json(&body);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(ClientError::Network(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )));
        }

        tokio::spawn(async move {
            if let Err(e) = Self::process_stream(response, sender).await {
                if let Some(log) = log_service {
                    log.error("a2a_streaming", &format!("Stream processing error: {e}"))
                        .await
                        .ok();
                }
            }
        });

        Ok(Self { receiver })
    }

    async fn process_stream(
        response: Response,
        sender: mpsc::Sender<ClientResult<StreamEvent>>,
    ) -> ClientResult<()> {
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let chunk_str = String::from_utf8_lossy(&chunk);
                    buffer.push_str(&chunk_str);

                    while let Some(line_end) = buffer.find('\n') {
                        let line = buffer[..line_end].trim_end_matches('\r').to_string();
                        buffer = buffer[line_end + 1..].to_string();

                        if let Some(event) = Self::parse_sse_line(&line)? {
                            if sender.send(Ok(event)).await.is_err() {
                                return Ok(());
                            }
                        }
                    }
                },
                Err(e) => {
                    let _ = sender
                        .send(Err(ClientError::stream(format!(
                            "Stream read error: {}",
                            e
                        ))))
                        .await;
                    return Ok(());
                },
            }
        }

        let _ = sender.send(Ok(StreamEvent::Complete)).await;
        Ok(())
    }

    fn parse_sse_line(line: &str) -> ClientResult<Option<StreamEvent>> {
        if line.is_empty() || line.starts_with(':') {
            return Ok(None);
        }

        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Ok(Some(StreamEvent::Complete));
            }

            match serde_json::from_str::<Value>(data) {
                Ok(json) => {
                    if let Some(content) = json.get("content").and_then(|v| v.as_str()) {
                        Ok(Some(StreamEvent::Content(content.to_string())))
                    } else if json.get("tool_use").is_some() {
                        Ok(Some(StreamEvent::Tool(json)))
                    } else if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
                        Ok(Some(StreamEvent::Error(error.to_string())))
                    } else {
                        Ok(None)
                    }
                },
                Err(_) => Ok(Some(StreamEvent::Content(data.to_string()))),
            }
        } else {
            Ok(None)
        }
    }
}

impl Stream for SseStream {
    type Item = ClientResult<StreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
