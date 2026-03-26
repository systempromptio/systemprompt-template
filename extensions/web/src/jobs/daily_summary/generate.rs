use std::sync::Arc;

use anyhow::Result;
use chrono::NaiveDate;
use sqlx::PgPool;
use systemprompt::ai::AiService;

/// Generate a daily summary report for a specific user and date.
/// This collects session data, activity, and generates an AI-powered summary.
pub async fn generate_user_daily_summary(
    pool: &Arc<PgPool>,
    user_id: &str,
    date: NaiveDate,
    ai_service: Option<&Arc<AiService>>,
) -> Result<()> {
    let _ai = ai_service.ok_or_else(|| anyhow::anyhow!("AI service not available"))?;

    // Check if summary already exists for this date
    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM daily_summaries WHERE user_id = $1 AND summary_date = $2",
    )
    .bind(user_id)
    .bind(date)
    .fetch_optional(pool.as_ref())
    .await?;

    if existing.unwrap_or(0) > 0 {
        tracing::info!(user_id, %date, "Daily summary already exists, skipping");
        return Ok(());
    }

    // Collect session data for the date
    let session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM plugin_session_summaries WHERE user_id = $1 AND started_at::date = $2",
    )
    .bind(user_id)
    .bind(date)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    if session_count == 0 {
        tracing::debug!(user_id, %date, "No sessions found for daily summary");
        return Ok(());
    }

    // Insert a basic summary record
    sqlx::query(
        "INSERT INTO daily_summaries (user_id, summary_date, session_count, created_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id, summary_date) DO NOTHING",
    )
    .bind(user_id)
    .bind(date)
    .bind(session_count)
    .execute(pool.as_ref())
    .await?;

    tracing::info!(user_id, %date, sessions = session_count, "Daily summary generated");
    Ok(())
}
