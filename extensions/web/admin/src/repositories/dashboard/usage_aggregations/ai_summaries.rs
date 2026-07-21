use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

pub async fn update_session_ai_summary_structured(
    pool: &PgPool,
    session_id: &SessionId,
    summary: &crate::handlers::hooks_track::ai_summary::SessionAnalysis,
) {
    let tags = summary.tags.join(",");
    let composed_summary = summary.composed_summary();
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET ai_title = $2, ai_description = $3, ai_summary = $4, ai_tags = $5, updated_at = NOW()
          WHERE session_id = $1",
        session_id.as_str(),
        summary.title,
        summary.description,
        composed_summary,
        tags,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update structured session AI summary");
    }
}
