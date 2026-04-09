use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::error::MarketplaceError;

pub struct UsageEventParams<'a> {
    pub user_id: &'a UserId,
    pub session_id: &'a SessionId,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub metadata: &'a serde_json::Value,
    pub description: Option<&'a str>,
    pub prompt_preview: Option<&'a str>,
    pub cwd: Option<&'a str>,
    pub dedup_key: &'a str,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
}

pub async fn insert_plugin_usage_event(
    pool: &PgPool,
    params: &UsageEventParams<'_>,
) -> Result<bool, MarketplaceError> {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO plugin_usage_events
            (id, user_id, session_id, event_type, tool_name, metadata,
             description, prompt_preview, cwd, dedup_key)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (dedup_key) WHERE dedup_key IS NOT NULL DO NOTHING",
        &id,
        params.user_id.as_str(),
        params.session_id.as_str(),
        params.event_type,
        params.tool_name,
        params.metadata,
        params.description,
        params.prompt_preview,
        params.cwd,
        params.dedup_key,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
