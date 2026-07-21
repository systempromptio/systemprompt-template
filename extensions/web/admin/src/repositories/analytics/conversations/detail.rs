//! Single-conversation detail + raw-turn queries.
//!
//! `fetch_conversation_detail` joins the latest transcript with the session
//! summary, the per-session governance decisions, and a representative
//! `trace_id`, then normalises the transcript JSONB into `TranscriptTurn`s.
//! `find_raw_turns` backs the capability-gated PII endpoint.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use super::RawTurnBody;
use super::transcript::extract_content_text;

pub async fn find_raw_turns(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Option<Vec<RawTurnBody>>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT transcript AS "transcript!"
           FROM session_transcripts
           WHERE session_id = $1
           ORDER BY captured_at DESC
           LIMIT 1"#,
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let arr = row.transcript.as_array().cloned().unwrap_or_default();
    let raw = arr
        .iter()
        .enumerate()
        .map(|(i, entry)| RawTurnBody {
            ordinal: i32::try_from(i).unwrap_or(i32::MAX),
            content: extract_content_text(entry),
        })
        .collect();
    Ok(Some(raw))
}
