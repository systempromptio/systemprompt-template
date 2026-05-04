use sqlx::PgPool;

#[derive(Debug, Clone, Copy)]
pub struct AchievementCounts {
    pub session_count: i64,
    pub tool_count: i64,
    pub custom_skills_count: i64,
    pub error_count: i64,
}

pub async fn fetch_achievement_counts(
    pool: &PgPool,
    user_id: &str,
) -> Result<AchievementCounts, sqlx::Error> {
    let session_count: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_SessionStart'",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let tool_count: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_PostToolUse'",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let custom_skills_count: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM user_skills WHERE user_id = $1",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let error_count: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_PostToolUseFailure'",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    Ok(AchievementCounts {
        session_count,
        tool_count,
        custom_skills_count,
        error_count,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct TimeBasedFlags {
    pub has_early: bool,
    pub has_late: bool,
    pub has_weekend: bool,
}

pub async fn fetch_time_based_flags(
    pool: &PgPool,
    user_id: &str,
) -> Result<TimeBasedFlags, sqlx::Error> {
    let has_early: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM plugin_usage_events WHERE user_id = $1 AND EXTRACT(HOUR FROM created_at) < 7)",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    let has_late: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM plugin_usage_events WHERE user_id = $1 AND EXTRACT(HOUR FROM created_at) >= 22)",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    let has_weekend: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM plugin_usage_events WHERE user_id = $1 AND EXTRACT(DOW FROM created_at) IN (0, 6))",
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    Ok(TimeBasedFlags {
        has_early,
        has_late,
        has_weekend,
    })
}

pub async fn insert_achievements(
    pool: &PgPool,
    user_id: &str,
    to_unlock: &[&str],
) -> Result<(), sqlx::Error> {
    for achievement_id in to_unlock {
        sqlx::query!(
            "INSERT INTO employee_achievements (id, user_id, achievement_id) VALUES (gen_random_uuid()::TEXT, $1, $2) ON CONFLICT (user_id, achievement_id) DO NOTHING",
            user_id,
            *achievement_id,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
