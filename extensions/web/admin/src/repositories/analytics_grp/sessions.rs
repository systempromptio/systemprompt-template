use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub events: i64,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
}

pub async fn list_sessions(pool: &PgPool) -> Result<Vec<SessionRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionRow,
        r#"SELECT
            session_id AS "session_id!: SessionId",
            MAX(user_id) AS "user_id?: UserId",
            COUNT(*)::bigint AS "events!",
            MIN(created_at) AS "first_seen?",
            MAX(created_at) AS "last_seen?"
          FROM plugin_usage_events
          WHERE session_id IS NOT NULL
            AND created_at >= NOW() - INTERVAL '24 hours'
          GROUP BY session_id
          ORDER BY MAX(created_at) DESC
          LIMIT 50"#,
    )
    .fetch_all(pool)
    .await
}
