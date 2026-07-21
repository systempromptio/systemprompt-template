//! Per-user usage event rollups.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::UserUsageEvent;

pub async fn get_user_usage(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserUsageEvent>, sqlx::Error> {
    sqlx::query_as!(
        UserUsageEvent,
        r#"
        SELECT id, event_type, tool_name, created_at, metadata
        FROM plugin_usage_events
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
