use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{ClientId, ClientType};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClientAnalyticsRow {
    pub client_id: String,
    pub session_count: i64,
    pub unique_users: i64,
    pub total_requests: i64,
    pub total_tokens: i64,
    pub total_cost_cents: i64,
    pub avg_session_duration_seconds: f64,
    pub avg_response_time_ms: f64,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClientErrorAnalyticsRow {
    pub client_id: String,
    pub error_count: i64,
    pub affected_sessions: i64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientErrorAnalytics {
    pub client_id: ClientId,
    pub error_count: i64,
    pub affected_sessions: i64,
    pub last_error: String,
}

impl From<ClientAnalyticsRow> for ClientAnalytics {
    fn from(row: ClientAnalyticsRow) -> Self {
        let client_id = ClientId::new(&row.client_id);
        let client_type = client_id.client_type();
        Self {
            client_id,
            client_type,
            session_count: row.session_count,
            unique_users: row.unique_users,
            total_requests: row.total_requests,
            total_tokens: row.total_tokens,
            total_cost_cents: row.total_cost_cents,
            avg_session_duration_seconds: row.avg_session_duration_seconds,
            avg_response_time_ms: row.avg_response_time_ms,
            first_seen: row.first_seen.to_rfc3339(),
            last_seen: row.last_seen.to_rfc3339(),
        }
    }
}

impl From<ClientErrorAnalyticsRow> for ClientErrorAnalytics {
    fn from(row: ClientErrorAnalyticsRow) -> Self {
        Self {
            client_id: ClientId::new(&row.client_id),
            error_count: row.error_count,
            affected_sessions: row.affected_sessions,
            last_error: row.last_error.unwrap_or_default(),
        }
    }
}
