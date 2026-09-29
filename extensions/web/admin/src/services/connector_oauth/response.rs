//! Decoding of provider HTTP and JSON-RPC responses during verification,
//! with credentials scrubbed from any error text that reaches a caller.

use super::Grant;
use crate::error::{AdminError, AdminResult};
// JSON: JSON-RPC and provider payloads are an open protocol boundary; only the
// `result`, `error` and `isError` members are read before the typed decoders.
use serde_json::Value;

pub(super) async fn body(response: reqwest::Response) -> AdminResult<Value> {
    let status = response.status();
    if status.as_u16() == 401 {
        return Err(AdminError::Unauthorized("Provider grant rejected".into()));
    }
    if status.as_u16() == 403 {
        return Err(AdminError::Forbidden(
            "Provider permissions do not allow this operation".into(),
        ));
    }
    if !status.is_success() {
        return Err(AdminError::Upstream(
            "Provider temporarily unavailable".into(),
        ));
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_redacted_error| AdminError::Upstream("Provider response interrupted".into()))?
    {
        if bytes.len() + chunk.len() > 1_048_576 {
            return Err(AdminError::Upstream(
                "Provider response exceeds verification limit".into(),
            ));
        }
        bytes.extend_from_slice(&chunk);
        if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
            return Ok(value);
        }
        // Why: Streamable HTTP can answer a request on an SSE stream that remains
        // open. Return when its JSON-RPC response arrives, not at stream EOF.
        if let Ok(text) = std::str::from_utf8(&bytes) {
            for frame in text.split("\n\n") {
                let data = frame
                    .lines()
                    .filter_map(|l| l.strip_prefix("data:"))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Ok(value) = serde_json::from_str::<Value>(&data)
                    && value.get("id").is_some()
                {
                    return Ok(value);
                }
            }
        }
    }
    Err(AdminError::Upstream(
        "Provider returned an invalid response".into(),
    ))
}

pub(super) fn tool_error(grant: &Grant, payload: &Value, value: &Value) -> AdminError {
    let operation = payload
        .pointer("/params/name")
        .and_then(Value::as_str)
        .or_else(|| payload.get("method").and_then(Value::as_str))
        .unwrap_or("request");
    let mut detail = value
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .pointer("/result/content/0/text")
                .and_then(Value::as_str)
        })
        .unwrap_or("Provider returned a tool error")
        .to_owned();
    for secret in [
        &grant.access_token,
        grant.refresh_token.as_deref().unwrap_or(""),
        &grant.client_secret,
        &grant.verifier,
    ] {
        if !secret.is_empty() {
            detail = detail.replace(secret, "[redacted]");
        }
    }
    let detail: String = detail
        .chars()
        .filter(|c| !c.is_control())
        .take(600)
        .collect();
    AdminError::Unavailable(format!(
        "{} MCP {operation}: {detail}",
        grant.provider.slug()
    ))
}
