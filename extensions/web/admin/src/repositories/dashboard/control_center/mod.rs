mod sessions;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::control_center::ActivityFeedEvent;
pub use crate::types::control_center::TodayStats;

pub use sessions::fetch_recent_sessions_filtered;

pub async fn fetch_session_events(
    pool: &PgPool,
    user_id: &UserId,
    session_ids: &[String],
) -> Result<Vec<ActivityFeedEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityFeedEvent,
        r"SELECT
            id, session_id, event_type, tool_name,
            description, prompt_preview, cwd,
            created_at
        FROM plugin_usage_events
        WHERE user_id = $1 AND session_id = ANY($2)
        ORDER BY created_at DESC",
        user_id.as_str(),
        session_ids,
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, Copy)]
pub struct TodayOutcomeStats {
    pub completed_today: i64,
    pub positive_count: i64,
    pub rated_count: i64,
}
