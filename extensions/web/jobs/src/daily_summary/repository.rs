//! Data-access layer for the daily-summary job.
//!
//! Keeps the compile-time `sqlx` queries out of [`super::generate`] so the job
//! logic depends on named repository methods rather than inline SQL.

use chrono::NaiveDate;
use sqlx::PgPool;

/// Count existing `daily_summaries` rows for a user on a given date.
///
/// Used to make summary generation idempotent — a non-zero count means the
/// summary already exists and generation is skipped.
pub async fn count_existing_summaries(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
) -> Result<i64, sqlx::Error> {
    let count: Option<i64> = sqlx::query_scalar!(
        "SELECT COUNT(*)::BIGINT FROM daily_summaries WHERE user_id = $1 AND summary_date = $2",
        user_id,
        date,
    )
    .fetch_optional(pool)
    .await?
    .flatten();
    Ok(count.unwrap_or(0))
}

/// Count plugin sessions recorded for a user on a given date.
pub async fn count_sessions_for_date(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
) -> Result<i64, sqlx::Error> {
    let count: Option<i64> = sqlx::query_scalar!(
        "SELECT COUNT(*)::BIGINT FROM plugin_session_summaries WHERE user_id = $1 AND started_at::date = $2",
        user_id,
        date,
    )
    .fetch_one(pool)
    .await?;
    Ok(count.unwrap_or(0))
}

/// Insert the computed daily summary row, ignoring an existing one.
pub async fn insert_daily_summary(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
    session_count: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO daily_summaries (user_id, summary_date, session_count, created_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id, summary_date) DO NOTHING",
        user_id,
        date,
        session_count,
    )
    .execute(pool)
    .await?;
    Ok(())
}
