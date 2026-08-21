//! Per-user daily usage records from `admin_usage_daily_rollups`.
//!
//! A straight PK-range read: the hourly `usage_daily_rollup` job maintains
//! this table, so the drill-down page's daily table reads one narrow row per
//! day instead of re-aggregating raw events — at the cost of up to an hour of
//! lag, which the page's caption states ("updated hourly").

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::util::time_range::TimeRange;

#[derive(Debug, Clone, Copy)]
pub struct UserDailyRollupRow {
    pub date: chrono::NaiveDate,
    pub sessions_count: i32,
    pub prompts: i64,
    pub tool_uses: i64,
    pub errors: i64,
    pub loc_added_ai: i64,
    pub loc_removed_ai: i64,
    pub commits_count: i32,
    pub commit_insertions: i64,
    pub commit_deletions: i64,
    pub ai_requests_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_microdollars: i64,
}

pub async fn list_user_daily_rollups(
    pool: &PgPool,
    user_id: &UserId,
    range: TimeRange,
) -> Result<Vec<UserDailyRollupRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT date AS "date!", sessions_count AS "sessions_count!",
               prompts AS "prompts!", tool_uses AS "tool_uses!",
               errors AS "errors!",
               loc_added_ai AS "loc_added_ai!", loc_removed_ai AS "loc_removed_ai!",
               commits_count AS "commits_count!",
               commit_insertions AS "commit_insertions!",
               commit_deletions AS "commit_deletions!",
               ai_requests_count AS "ai_requests_count!",
               input_tokens AS "input_tokens!", output_tokens AS "output_tokens!",
               cost_microdollars AS "cost_microdollars!"
        FROM admin_usage_daily_rollups
        WHERE user_id = $1 AND date >= $2::DATE AND date <= $3::DATE
        ORDER BY date DESC
        "#,
        user_id.as_str(),
        range.from.date_naive(),
        range.to.date_naive(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UserDailyRollupRow {
            date: r.date,
            sessions_count: r.sessions_count,
            prompts: r.prompts,
            tool_uses: r.tool_uses,
            errors: r.errors,
            loc_added_ai: r.loc_added_ai,
            loc_removed_ai: r.loc_removed_ai,
            commits_count: r.commits_count,
            commit_insertions: r.commit_insertions,
            commit_deletions: r.commit_deletions,
            ai_requests_count: r.ai_requests_count,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost_microdollars: r.cost_microdollars,
        })
        .collect())
}
