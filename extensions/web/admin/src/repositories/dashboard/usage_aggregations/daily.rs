//! Daily and per-session usage counters.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};


#[derive(Debug, Clone, Copy)]
pub struct DailyAggregationParams<'a> {
    pub pool: &'a PgPool,
    pub user_id: &'a UserId,
    pub date: &'a chrono::NaiveDate,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub loc_added: i64,
    pub loc_removed: i64,
    pub is_error: bool,
}

pub async fn upsert_daily_aggregation(params: &DailyAggregationParams<'_>) {
    let id = format!(
        "{date}_{user_id}_{}_{}",
        params.event_type,
        params.tool_name.unwrap_or(""),
        date = params.date,
        user_id = params.user_id.as_str(),
    );
    let error_inc = i64::from(params.is_error);

    let result = sqlx::query!(
        r"INSERT INTO plugin_usage_daily
            (id, date, user_id, event_type, tool_name, event_count, content_input_bytes, content_output_bytes, error_count, loc_added, loc_removed)
           VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $8, $9, $10)
           ON CONFLICT (date, user_id, event_type, COALESCE(tool_name, ''))
           DO UPDATE SET
             event_count = plugin_usage_daily.event_count + 1,
             content_input_bytes = plugin_usage_daily.content_input_bytes + $6,
             content_output_bytes = plugin_usage_daily.content_output_bytes + $7,
             error_count = plugin_usage_daily.error_count + $8,
             loc_added = plugin_usage_daily.loc_added + $9,
             loc_removed = plugin_usage_daily.loc_removed + $10,
             updated_at = NOW()",
        id,
        params.date,
        params.user_id.as_str(),
        params.event_type,
        params.tool_name,
        params.content_input_bytes,
        params.content_output_bytes,
        error_inc,
        params.loc_added,
        params.loc_removed,
    )
    .execute(params.pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to upsert daily aggregation");
    }
}

// Why: totals come from durable accepted events, so recovery cannot increment
// them twice.
pub async fn refresh_session_summary(
    pool: &PgPool,
    session_id: &SessionId,
    file_path: Option<&str>,
) {
    if let Err(error) = super::ingestion::drain_ingestion_outbox(pool).await {
        tracing::error!(%error, "Session aggregation remains queued for retry");
    }
    if let Some(path) = file_path.filter(|path| !path.is_empty()) {
        super::session_updates::update_unique_files_touched(pool, session_id, path).await;
    }
}
