use anyhow::Result;
use reqwest::Response;
use serde::de::DeserializeOwned;

pub async fn extract_session_from_cookie(response: &Response) -> Option<String> {
    response
        .cookies()
        .find(|c| c.name() == "access_token")
        .map(|c| decode_jwt_session_id(c.value()))
}

fn decode_jwt_session_id(token: &str) -> String {
    use base64::engine::general_purpose;
    use base64::Engine as _;

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return "invalid_token".to_string();
    }

    match general_purpose::STANDARD.decode(parts[1]) {
        Ok(decoded) => {
            if let Ok(payload) = String::from_utf8(decoded) {
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&payload) {
                    if let Some(session_id) = json_val.get("session_id").and_then(|v| v.as_str()) {
                        return session_id.to_string();
                    }
                }
            }
            "decode_failed".to_string()
        },
        Err(_) => "decode_failed".to_string(),
    }
}

pub struct StreamChunk {
    pub delta: String,
    pub task_id: Option<String>,
}

pub async fn parse_sse_stream(response: Response) -> Result<Vec<StreamChunk>> {
    use futures::stream::StreamExt;

    let mut chunks = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            if line.starts_with("data: ") {
                let json = &line[6..];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json) {
                    let delta = parsed
                        .get("delta")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let task_id = parsed
                        .get("task_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    chunks.push(StreamChunk { delta, task_id });
                }
            }
        }
    }

    Ok(chunks)
}

pub async fn parse_json_response<T: DeserializeOwned>(response: Response) -> Result<T> {
    let text = response.text().await?;
    Ok(serde_json::from_str(&text)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_jwt_session_id() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
                     eyJzZXNzaW9uX2lkIjoic2Vzc18xMjM0NTY3OCJ9.signature";
        let session_id = decode_jwt_session_id(token);
        assert_eq!(session_id, "sess_12345678");
    }

    #[test]
    fn test_decode_jwt_invalid_token() {
        let token = "invalid";
        let session_id = decode_jwt_session_id(token);
        assert_eq!(session_id, "invalid_token");
    }
}
