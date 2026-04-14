use sqlx::PgPool;

use super::{ERROR_XP, PROMPT_XP, SESSION_XP, SUBAGENT_XP, TOKEN_XP_PER_1K, TOOL_USE_XP};

pub(super) type UserXpResult = (i64, i64, i32, i32, i64, i64, i64, i32);

pub(super) struct UserRankParams<'a> {
    pub pool: &'a PgPool,
    pub uid: &'a str,
    pub total_xp: i64,
    pub rank_level: i32,
    pub rank_name: &'a str,
    pub events_count: i64,
    pub unique_skills: i32,
    pub unique_plugins: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_active_date: Option<chrono::NaiveDate>,
}

pub(super) async fn populate_daily_usage(pool: &PgPool) -> Result<(), super::GamificationError> {
    sqlx::query!(
        r"
        INSERT INTO employee_daily_usage (user_id, usage_date, event_count)
        SELECT e.user_id, DATE(e.created_at), COUNT(*)::INT
        FROM plugin_usage_events e
        INNER JOIN users u ON u.id = e.user_id
        GROUP BY e.user_id, DATE(e.created_at)
        ON CONFLICT (user_id, usage_date) DO UPDATE SET event_count = EXCLUDED.event_count
        ",
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(super) async fn calculate_user_xp(
    pool: &PgPool,
    uid: &str,
) -> Result<UserXpResult, super::GamificationError> {
    let base_xp: i64 = sqlx::query_scalar!(
        r"
        SELECT COALESCE(SUM(
            CASE
                WHEN event_type = 'claude_code_SessionStart' THEN $2
                WHEN event_type = 'claude_code_PostToolUse' THEN $3
                WHEN event_type = 'claude_code_PostToolUseFailure' THEN $4
                WHEN event_type = 'claude_code_UserPromptSubmit' THEN $5
                WHEN event_type = 'claude_code_SubagentStart' THEN $6
                ELSE 0
            END
        ), 0)::BIGINT
        FROM plugin_usage_events
        WHERE user_id = $1
        ",
        uid,
        SESSION_XP,
        TOOL_USE_XP,
        ERROR_XP,
        PROMPT_XP,
        SUBAGENT_XP,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let total_tokens: i64 = sqlx::query_scalar!(
        r"
        SELECT COALESCE(SUM(
            COALESCE((metadata->>'input_tokens')::BIGINT, 0) +
            COALESCE((metadata->>'output_tokens')::BIGINT, 0)
        ), 0)::BIGINT
        FROM plugin_usage_events
        WHERE user_id = $1
        ",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let token_xp = (total_tokens / 1000) * i64::from(TOKEN_XP_PER_1K);

    let bonus_xp: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(xp_amount), 0)::BIGINT FROM employee_xp_ledger WHERE user_id = $1",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let total_xp = base_xp + token_xp + bonus_xp;

    let events_count: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let unique_skills: i32 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(DISTINCT tool_name), 0)::INT FROM plugin_usage_events WHERE user_id = $1 AND tool_name IS NOT NULL",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let unique_plugins: i32 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(DISTINCT plugin_id), 0)::INT FROM plugin_usage_events WHERE user_id = $1 AND plugin_id IS NOT NULL",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let prompt_count: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_UserPromptSubmit'",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let subagent_count: i64 = sqlx::query_scalar!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events WHERE user_id = $1 AND event_type = 'claude_code_SubagentStart'",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let models_used: i32 = sqlx::query_scalar!(
        r"
        SELECT COALESCE(COUNT(DISTINCT metadata->>'model'), 0)::INT
        FROM plugin_usage_events
        WHERE user_id = $1
          AND event_type = 'claude_code_SessionStart'
          AND metadata->>'model' IS NOT NULL
        ",
        uid,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    Ok((
        total_xp,
        events_count,
        unique_skills,
        unique_plugins,
        total_tokens,
        prompt_count,
        subagent_count,
        models_used,
    ))
}

pub(super) async fn calculate_streaks(
    pool: &PgPool,
    uid: &str,
) -> Result<(i32, i32, Option<chrono::NaiveDate>), super::GamificationError> {
    #[derive(sqlx::FromRow)]
    struct DateRow {
        usage_date: chrono::NaiveDate,
    }

    let rows = sqlx::query_as!(
        DateRow,
        "SELECT usage_date FROM employee_daily_usage WHERE user_id = $1 ORDER BY usage_date DESC",
        uid,
    )
    .fetch_all(pool)
    .await?;

    let dates: Vec<chrono::NaiveDate> = rows.iter().map(|d| d.usage_date).collect();
    let last_active_date = dates.first().copied();
    let current_streak = compute_current_streak(&dates);
    let longest_streak = compute_longest_streak(&dates);

    Ok((current_streak, longest_streak, last_active_date))
}

fn compute_current_streak(dates_desc: &[chrono::NaiveDate]) -> i32 {
    let today = chrono::Utc::now().date_naive();
    let mut current_streak = 0i32;
    let mut expected = today;
    for &usage in dates_desc {
        if usage == expected {
            current_streak += 1;
            expected -= chrono::Duration::days(1);
        } else if usage == expected - chrono::Duration::days(1) && current_streak == 0 {
            expected = usage;
            current_streak = 1;
            expected -= chrono::Duration::days(1);
        } else {
            break;
        }
    }
    current_streak
}

fn compute_longest_streak(dates_desc: &[chrono::NaiveDate]) -> i32 {
    let mut sorted: Vec<chrono::NaiveDate> = dates_desc.to_vec();
    sorted.sort();

    let mut longest = 0i32;
    let mut streak = 0i32;
    let mut prev: Option<chrono::NaiveDate> = None;

    for &date in &sorted {
        if let Some(p) = prev {
            if date == p + chrono::Duration::days(1) {
                streak += 1;
            } else {
                streak = 1;
            }
        } else {
            streak = 1;
        }
        if streak > longest {
            longest = streak;
        }
        prev = Some(date);
    }
    longest
}

pub(super) async fn update_user_rank(params: &UserRankParams<'_>) -> Result<(), super::GamificationError> {
    let total_xp_i32 = i32::try_from(params.total_xp).unwrap_or(i32::MAX);
    sqlx::query!(
        r"
        INSERT INTO employee_ranks (user_id, total_xp, rank_level, rank_name, events_count, unique_skills_count, unique_plugins_count, current_streak, longest_streak, last_active_date, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        ON CONFLICT (user_id) DO UPDATE SET
            total_xp = EXCLUDED.total_xp,
            rank_level = EXCLUDED.rank_level,
            rank_name = EXCLUDED.rank_name,
            events_count = EXCLUDED.events_count,
            unique_skills_count = EXCLUDED.unique_skills_count,
            unique_plugins_count = EXCLUDED.unique_plugins_count,
            current_streak = EXCLUDED.current_streak,
            longest_streak = GREATEST(employee_ranks.longest_streak, EXCLUDED.longest_streak),
            last_active_date = EXCLUDED.last_active_date,
            updated_at = NOW()
        ",
        params.uid,
        total_xp_i32,
        params.rank_level,
        params.rank_name,
        params.events_count,
        params.unique_skills,
        params.unique_plugins,
        params.current_streak,
        params.longest_streak,
        params.last_active_date,
    )
    .execute(params.pool)
    .await?;
    Ok(())
}
