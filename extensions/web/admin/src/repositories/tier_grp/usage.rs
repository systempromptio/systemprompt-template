use sqlx::PgPool;

#[derive(Debug, Clone, Copy)]
pub struct DailyUsageTotals {
    pub events: i64,
    pub bytes: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct EntityCounts {
    pub skills: i64,
    pub agents: i64,
    pub plugins: i64,
    pub mcp_servers: i64,
    pub hooks: i64,
}

pub async fn get_usage_today_totals(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<DailyUsageTotals>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COALESCE(SUM(event_count), 0)::BIGINT AS "events!",
            COALESCE(SUM(content_input_bytes + content_output_bytes), 0)::BIGINT AS "bytes!"
           FROM plugin_usage_daily
           WHERE user_id = $1 AND date = CURRENT_DATE"#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| DailyUsageTotals {
        events: r.events,
        bytes: r.bytes,
    }))
}

pub async fn get_session_count_today(pool: &PgPool, user_id: &str) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM plugin_session_summaries WHERE user_id = $1 AND started_at::date = CURRENT_DATE",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    Ok(count)
}

pub async fn get_entity_counts(pool: &PgPool, user_id: &str) -> Result<EntityCounts, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            (SELECT COUNT(*) FROM public.user_skills WHERE user_id = $1) AS "skills!",
            (SELECT COUNT(*) FROM public.user_agents WHERE user_id = $1) AS "agents!",
            (SELECT COUNT(*) FROM public.user_plugins WHERE user_id = $1) AS "plugins!",
            (SELECT COUNT(*) FROM public.user_mcp_servers WHERE user_id = $1) AS "mcp_servers!",
            (SELECT COUNT(*) FROM public.user_hooks WHERE user_id = $1) AS "hooks!""#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(EntityCounts {
        skills: row.skills,
        agents: row.agents,
        plugins: row.plugins,
        mcp_servers: row.mcp_servers,
        hooks: row.hooks,
    })
}
