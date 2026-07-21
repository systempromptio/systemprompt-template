//! Computes a session's actions-per-minute from its event timestamps.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::numeric;

pub async fn calculate_session_apm(pool: &PgPool, session_id: &SessionId) -> (f32, f32) {
    struct Row {
        tool_uses: Option<i64>,
        prompts: Option<i64>,
        errors: Option<i64>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let row = sqlx::query_as!(
        Row,
        r"SELECT tool_uses, prompts, errors, started_at, ended_at
          FROM plugin_session_summaries
          WHERE session_id = $1",
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, session_id = %session_id, "Failed to fetch session APM data"))
    .ok()
    .flatten();

    let Some(r) = row else {
        return (0.0, 0.0);
    };

    let tool_uses = r.tool_uses.unwrap_or(0);
    let prompts = r.prompts.unwrap_or(0);
    let errors = r.errors.unwrap_or(0);

    let duration_minutes = match (r.started_at, r.ended_at) {
        (Some(s), Some(e)) => {
            let mins = numeric::seconds_to_f64((e - s).num_seconds()) / 60.0;
            mins.max(1.0)
        },
        _ => 1.0,
    };

    let apm = numeric::to_f32_from_i64(tool_uses + prompts) / numeric::to_f32(duration_minutes);
    let eapm = numeric::to_f32_from_i64((tool_uses + prompts - errors).max(0))
        / numeric::to_f32(duration_minutes);

    (apm, eapm)
}
