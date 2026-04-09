use std::sync::Arc;

use chrono::NaiveDate;
use sqlx::PgPool;
use systemprompt::ai::AiService;

use crate::error::MarketplaceError;

pub async fn generate_user_daily_summary(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
    ai_service: Option<&Arc<AiService>>,
) -> anyhow::Result<()> {
    let _ai = ai_service.ok_or(MarketplaceError::Internal(
        "AI service not available".to_string(),
    ))?;

    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM daily_summaries WHERE user_id = $1 AND summary_date = $2",
    )
    .bind(user_id)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(MarketplaceError::Database)?;

    if existing.unwrap_or(0) > 0 {
        tracing::info!(user_id, %date, "Daily summary already exists, skipping");
        return Ok(());
    }

    let session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM plugin_session_summaries WHERE user_id = $1 AND started_at::date = $2",
    )
    .bind(user_id)
    .bind(date)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if session_count == 0 {
        tracing::debug!(user_id, %date, "No sessions found for daily summary");
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO daily_summaries (user_id, summary_date, session_count, created_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (user_id, summary_date) DO NOTHING",
    )
    .bind(user_id)
    .bind(date)
    .bind(session_count)
    .execute(pool)
    .await
    .map_err(MarketplaceError::Database)?;

    tracing::info!(user_id, %date, sessions = session_count, "Daily summary generated");
    Ok(())
}
