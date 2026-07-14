use std::sync::Arc;

use chrono::NaiveDate;
use sqlx::PgPool;
use systemprompt::ai::AiService;

use systemprompt_web_shared::error::MarketplaceError;

use super::repository;

pub async fn generate_user_daily_summary(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
    ai_service: Option<&Arc<AiService>>,
) -> Result<(), MarketplaceError> {
    let _ai = ai_service.ok_or(MarketplaceError::Internal(
        "AI service not available".to_owned(),
    ))?;

    let existing = repository::count_existing_summaries(pool, user_id, date)
        .await
        .map_err(MarketplaceError::Database)?;

    if existing > 0 {
        tracing::info!(user_id, %date, "Daily summary already exists, skipping");
        return Ok(());
    }

    let session_count = repository::count_sessions_for_date(pool, user_id, date)
        .await
        .unwrap_or(0);

    if session_count == 0 {
        tracing::debug!(user_id, %date, "No sessions found for daily summary");
        return Ok(());
    }

    let session_count_i32 = i32::try_from(session_count).unwrap_or(i32::MAX);
    repository::insert_daily_summary(pool, user_id, date, session_count_i32)
        .await
        .map_err(MarketplaceError::Database)?;

    tracing::info!(user_id, %date, sessions = session_count, "Daily summary generated");
    Ok(())
}
