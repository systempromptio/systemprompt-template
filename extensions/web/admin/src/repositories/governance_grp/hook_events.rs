use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Serialize)]
pub struct RecentHookEvent {
    pub kind: String,
    pub created_at: DateTime<Utc>,
    pub plugin_id: Option<String>,
    pub tool_name: Option<String>,
    pub user_id: UserId,
    pub status: Option<String>,
}

pub async fn count_pretool_fired_24h(pool: &PgPool) -> Result<i64, MarketplaceError> {
    let row = sqlx::query!(
        "SELECT COUNT(*)::BIGINT AS n FROM governance_decisions \
         WHERE created_at > NOW() - INTERVAL '24 hours'",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.n.unwrap_or(0))
}

pub async fn count_posttool_fired_24h(pool: &PgPool) -> Result<i64, MarketplaceError> {
    let row = sqlx::query!(
        "SELECT COUNT(*)::BIGINT AS n FROM plugin_usage_events \
         WHERE created_at > NOW() - INTERVAL '24 hours'",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.n.unwrap_or(0))
}

pub async fn recent_hook_events(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RecentHookEvent>, MarketplaceError> {
    let pre = sqlx::query!(
        "SELECT created_at, plugin_id, tool_name, user_id AS \"user_id!: UserId\", decision \
         FROM governance_decisions \
         ORDER BY created_at DESC LIMIT $1",
        limit,
    )
    .fetch_all(pool)
    .await?;

    let post = sqlx::query!(
        "SELECT created_at, plugin_id, tool_name, user_id AS \"user_id!: UserId\", event_type \
         FROM plugin_usage_events \
         ORDER BY created_at DESC LIMIT $1",
        limit,
    )
    .fetch_all(pool)
    .await?;

    let mut merged: Vec<RecentHookEvent> = Vec::with_capacity(pre.len() + post.len());
    for r in pre {
        merged.push(RecentHookEvent {
            kind: "PreToolUse".to_owned(),
            created_at: r.created_at,
            plugin_id: r.plugin_id,
            tool_name: Some(r.tool_name),
            user_id: r.user_id,
            status: Some(r.decision),
        });
    }
    for r in post {
        merged.push(RecentHookEvent {
            kind: r.event_type,
            created_at: r.created_at,
            plugin_id: r.plugin_id,
            tool_name: r.tool_name,
            user_id: r.user_id,
            status: None,
        });
    }
    merged.sort_by_key(|e| std::cmp::Reverse(e.created_at));
    merged.truncate(usize::try_from(limit).unwrap_or(50));
    Ok(merged)
}
