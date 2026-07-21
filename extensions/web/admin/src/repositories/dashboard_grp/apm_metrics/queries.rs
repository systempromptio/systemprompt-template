use sqlx::PgPool;
use systemprompt::identifiers::SessionId;


pub async fn update_session_apm(
    pool: &PgPool,
    session_id: &SessionId,
    apm: f32,
    eapm: f32,
    peak_concurrent: i32,
) {
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET apm = $1, eapm = $2, peak_concurrent = $3, updated_at = NOW()
          WHERE session_id = $4",
        apm,
        eapm,
        peak_concurrent,
        session_id.as_str(),
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, session_id = %session_id, "Failed to update session APM");
    }
}
