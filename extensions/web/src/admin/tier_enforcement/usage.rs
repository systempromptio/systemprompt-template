use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::super::tier_limits::UsageSnapshot;

#[derive(sqlx::FromRow)]
struct DailyUsageRow {
    events: i64,
    bytes: i64,
}

pub async fn fetch_usage_from_db(pool: &PgPool, user_id: &UserId) -> UsageSnapshot {
    #[derive(sqlx::FromRow)]
    struct EntityCounts {
        skills: i64,
        agents: i64,
        plugins: i64,
        mcp_servers: i64,
        hooks: i64,
    }

    let uid = user_id.as_str();

    let daily: Option<DailyUsageRow> = sqlx::query_as(
        r"SELECT
            COALESCE(SUM(event_count), 0) AS events,
            COALESCE(SUM(content_input_bytes + content_output_bytes), 0) AS bytes
           FROM plugin_usage_daily
           WHERE user_id = $1 AND date = CURRENT_DATE",
    )
    .bind(uid)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let sessions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM plugin_session_summaries WHERE user_id = $1 AND started_at::date = CURRENT_DATE",
    )
    .bind(uid)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let counts: EntityCounts = sqlx::query_as(
        r"SELECT
            (SELECT COUNT(*) FROM marketplace.user_skills WHERE user_id = $1) AS skills,
            (SELECT COUNT(*) FROM marketplace.user_agents WHERE user_id = $1) AS agents,
            (SELECT COUNT(*) FROM marketplace.user_plugins WHERE user_id = $1) AS plugins,
            (SELECT COUNT(*) FROM marketplace.user_mcp_servers WHERE user_id = $1) AS mcp_servers,
            (SELECT COUNT(*) FROM marketplace.user_hooks WHERE user_id = $1) AS hooks",
    )
    .bind(uid)
    .fetch_one(pool)
    .await
    .unwrap_or(EntityCounts {
        skills: 0,
        agents: 0,
        plugins: 0,
        mcp_servers: 0,
        hooks: 0,
    });

    UsageSnapshot {
        events_today: daily.as_ref().map_or(0, |d| d.events),
        content_bytes_today: daily.as_ref().map_or(0, |d| d.bytes),
        sessions_today: sessions,
        skills_count: counts.skills,
        agents_count: counts.agents,
        plugins_count: counts.plugins,
        mcp_servers_count: counts.mcp_servers,
        hooks_count: counts.hooks,
    }
}
