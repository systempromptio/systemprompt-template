use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::super::tier_limits::UsageSnapshot;
use crate::repositories::tier_grp;

pub async fn fetch_usage_from_db(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<UsageSnapshot, sqlx::Error> {
    let uid = user_id.as_str();

    let daily = tier_grp::get_usage_today_totals(pool, uid).await?;
    let sessions = tier_grp::get_session_count_today(pool, uid).await?;
    let counts = tier_grp::get_entity_counts(pool, uid).await?;

    Ok(UsageSnapshot {
        events_today: daily.as_ref().map_or(0, |d| d.events),
        content_bytes_today: daily.as_ref().map_or(0, |d| d.bytes),
        sessions_today: sessions,
        skills_count: counts.skills,
        agents_count: counts.agents,
        plugins_count: counts.plugins,
        mcp_servers_count: counts.mcp_servers,
        hooks_count: counts.hooks,
    })
}
