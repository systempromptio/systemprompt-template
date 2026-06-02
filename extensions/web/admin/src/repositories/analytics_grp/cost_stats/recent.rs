//! Most-recent `ai_requests` rows, enriched with display name and department.

use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize)]
pub struct RecentRequest {
    pub id: String,
    pub user_id: String,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub context_id: Option<String>,
    pub display_name: Option<String>,
    pub department: Option<String>,
    pub model: String,
    pub status: String,
    pub error_message: Option<String>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn fetch_recent_requests(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RecentRequest>, sqlx::Error> {
    sqlx::query_as!(
        RecentRequest,
        r#"SELECT r.id, r.user_id, r.trace_id, r.session_id, r.context_id, r.model, r.status,
                  r.error_message, r.cost_microdollars, r.latency_ms, r.created_at,
                  u.display_name, upe.department
           FROM ai_requests r
           LEFT JOIN users u ON u.id = r.user_id
           LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
           ORDER BY r.created_at DESC
           LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await
}
