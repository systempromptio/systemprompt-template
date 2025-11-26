use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;
use systemprompt_identifiers::{ClientId, ClientType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAnalytics {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub session_count: i64,
    pub unique_users: i64,
    pub total_requests: i64,
    pub total_tokens: i64,
    pub total_cost_cents: i64,
    pub avg_session_duration_seconds: f64,
    pub avg_response_time_ms: f64,
    pub first_seen: String,
    pub last_seen: String,
}

impl ClientAnalytics {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let client_id_str = row
            .get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_id"))?;
        let client_id = ClientId::new(client_id_str);

        let client_type = client_id.client_type();

        Ok(Self {
            client_id,
            client_type,
            session_count: row
                .get("session_count")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
            unique_users: row
                .get("unique_users")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
            total_requests: row
                .get("total_requests")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
            total_tokens: row
                .get("total_tokens")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
            total_cost_cents: row
                .get("total_cost_cents")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
            avg_session_duration_seconds: row
                .get("avg_session_duration_seconds")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            avg_response_time_ms: row
                .get("avg_response_time_ms")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            first_seen: row
                .get("first_seen")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            last_seen: row
                .get("last_seen")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientErrorAnalytics {
    pub client_id: ClientId,
    pub error_count: i64,
    pub affected_sessions: i64,
    pub last_error: String,
}

impl ClientErrorAnalytics {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let client_id_str = row
            .get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_id"))?;

        Ok(Self {
            client_id: ClientId::new(client_id_str),
            error_count: row.get("error_count").and_then(serde_json::Value::as_i64).unwrap_or(0),
            affected_sessions: row
                .get("affected_sessions")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0),
            last_error: row
                .get("last_error")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}
