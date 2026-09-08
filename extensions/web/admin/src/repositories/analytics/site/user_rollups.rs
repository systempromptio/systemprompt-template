//! Per-user daily usage records from `admin_usage_daily_rollups`.
//!
//! A straight PK-range read: the hourly `usage_daily_rollup` job maintains
//! this table, so the drill-down page's daily table reads one narrow row per
//! day instead of re-aggregating raw events — at the cost of up to an hour of
//! lag, which the page's caption states ("updated hourly").


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
