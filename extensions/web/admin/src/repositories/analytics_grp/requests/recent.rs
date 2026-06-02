//! Lightweight recent gateway-request feed (no filters, newest first).

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct RecentGatewayRequestRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub trace_id: Option<String>,
    pub error_message: Option<String>,
}

pub async fn list_recent_gateway_requests(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RecentGatewayRequestRow>, sqlx::Error> {
    sqlx::query_as!(
        RecentGatewayRequestRow,
        r#"SELECT
            id as "id!",
            created_at as "created_at!",
            provider as "provider!",
            model as "model!",
            status as "status!",
            input_tokens,
            output_tokens,
            COALESCE(cost_microdollars, 0)::bigint as "cost_microdollars!",
            latency_ms,
            trace_id,
            error_message
          FROM ai_requests
          ORDER BY created_at DESC
          LIMIT $1"#,
        limit,
    )
    .fetch_all(pool)
    .await
}
