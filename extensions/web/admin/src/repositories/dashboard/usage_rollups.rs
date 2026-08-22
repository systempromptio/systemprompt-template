//! Set-based recompute of `admin_usage_daily_rollups`.
//!
//! Four idempotent upserts — hook-plane counters, session counts, observed
//! commits, and gateway requests — each recomputing a trailing window of
//! whole days so re-running never double-counts. Dates are UTC to match the
//! hook path's `Utc::now().date_naive()` bucketing. The prompt/tool/error
//! predicates mirror `increment_session_summary`'s classification, so the
//! rollup and the session summaries cannot disagree about what an event is.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

// Why: Recompute rollup rows for every UTC day in the trailing window
// (`days_back = 1` covers yesterday and today). Returns rows written.
pub async fn upsert_daily_rollups_for_window(
    pool: &PgPool,
    days_back: i32,
) -> Result<u64, MarketplaceError> {
    let mut written = 0u64;
    written += rollup_hook_counters(pool, days_back).await?;
    written += rollup_sessions(pool, days_back).await?;
    written += rollup_commits(pool, days_back).await?;
    written += rollup_gateway_requests(pool, days_back).await?;
    Ok(written)
}

async fn rollup_hook_counters(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO admin_usage_daily_rollups
            (user_id, date, prompts, tool_uses, errors, loc_added_ai, loc_removed_ai)
        SELECT
            user_id,
            date,
            COALESCE(SUM(event_count) FILTER (WHERE event_type LIKE '%UserPromptSubmit%'), 0),
            COALESCE(SUM(event_count)
                FILTER (WHERE event_type IN ('PostToolUse', 'PostToolUseFailure')), 0),
            COALESCE(SUM(error_count), 0),
            COALESCE(SUM(loc_added), 0),
            COALESCE(SUM(loc_removed), 0)
        FROM plugin_usage_daily
        WHERE date >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        GROUP BY user_id, date
        ON CONFLICT (user_id, date) DO UPDATE SET
            prompts = EXCLUDED.prompts,
            tool_uses = EXCLUDED.tool_uses,
            errors = EXCLUDED.errors,
            loc_added_ai = EXCLUDED.loc_added_ai,
            loc_removed_ai = EXCLUDED.loc_removed_ai,
            updated_at = NOW()
        "#,
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

async fn rollup_sessions(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO admin_usage_daily_rollups (user_id, date, sessions_count)
        SELECT
            user_id,
            (started_at AT TIME ZONE 'UTC')::DATE,
            COUNT(*)::INT
        FROM plugin_session_summaries
        WHERE started_at IS NOT NULL
          AND (started_at AT TIME ZONE 'UTC')::DATE >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        GROUP BY user_id, (started_at AT TIME ZONE 'UTC')::DATE
        ON CONFLICT (user_id, date) DO UPDATE SET
            sessions_count = EXCLUDED.sessions_count,
            updated_at = NOW()
        "#,
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

async fn rollup_commits(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO admin_usage_daily_rollups
            (user_id, date, commits_count, commit_insertions, commit_deletions)
        SELECT
            user_id,
            (committed_at AT TIME ZONE 'UTC')::DATE,
            COUNT(*)::INT,
            COALESCE(SUM(insertions), 0)::BIGINT,
            COALESCE(SUM(deletions), 0)::BIGINT
        FROM user_commits
        WHERE (committed_at AT TIME ZONE 'UTC')::DATE >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        GROUP BY user_id, (committed_at AT TIME ZONE 'UTC')::DATE
        ON CONFLICT (user_id, date) DO UPDATE SET
            commits_count = EXCLUDED.commits_count,
            commit_insertions = EXCLUDED.commit_insertions,
            commit_deletions = EXCLUDED.commit_deletions,
            updated_at = NOW()
        "#,
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

async fn rollup_gateway_requests(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO admin_usage_daily_rollups
            (user_id, date, ai_requests_count, input_tokens, output_tokens, cost_microdollars)
        SELECT
            user_id,
            (created_at AT TIME ZONE 'UTC')::DATE,
            COUNT(*)::BIGINT,
            COALESCE(SUM(input_tokens), 0)::BIGINT,
            COALESCE(SUM(output_tokens), 0)::BIGINT,
            COALESCE(SUM(cost_microdollars), 0)::BIGINT
        FROM ai_requests
        WHERE NOT COALESCE(synthetic, FALSE)
          AND (created_at AT TIME ZONE 'UTC')::DATE >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        GROUP BY user_id, (created_at AT TIME ZONE 'UTC')::DATE
        ON CONFLICT (user_id, date) DO UPDATE SET
            ai_requests_count = EXCLUDED.ai_requests_count,
            input_tokens = EXCLUDED.input_tokens,
            output_tokens = EXCLUDED.output_tokens,
            cost_microdollars = EXCLUDED.cost_microdollars,
            updated_at = NOW()
        "#,
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}
