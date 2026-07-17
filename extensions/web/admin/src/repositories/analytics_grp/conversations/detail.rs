//! Single-conversation detail + raw-turn queries.
//!
//! `fetch_conversation_detail` joins the latest transcript with the session
//! summary, the per-session governance decisions, and a representative
//! `trace_id`, then normalises the transcript JSONB into `TranscriptTurn`s.
//! `fetch_raw_turns` backs the capability-gated PII endpoint.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId, TraceId, UserId};

use super::transcript::{GovernanceRow, ParseInput, extract_content_text, parse_turns};
use super::{ConversationDetail, RawTurnBody, TranscriptTurn};

struct DetailFields {
    user_id: Option<UserId>,
    plugin_id: Option<PluginId>,
    ai_title: Option<String>,
    ai_summary: Option<String>,
    model: Option<String>,
    started_at: Option<DateTime<Utc>>,
    ended_at: Option<DateTime<Utc>>,
}

fn assemble_detail(
    session_id: &SessionId,
    fields: DetailFields,
    turns: Vec<TranscriptTurn>,
) -> ConversationDetail {
    ConversationDetail {
        session_id: session_id.clone(),
        user_id: fields.user_id,
        plugin_id: fields.plugin_id,
        ai_title: fields.ai_title,
        ai_summary: fields.ai_summary,
        model: fields.model,
        started_at: fields.started_at,
        ended_at: fields.ended_at,
        turns,
    }
}

pub async fn fetch_conversation_detail(
    pool: &PgPool,
    session_id: &SessionId,
    include_raw: bool,
) -> Result<Option<ConversationDetail>, sqlx::Error> {
    let summary = sqlx::query!(
        r#"SELECT user_id AS "user_id: UserId", plugin_id AS "plugin_id: PluginId", ai_title, ai_summary, model, started_at, ended_at
          FROM plugin_session_summaries
          WHERE session_id = $1
          LIMIT 1"#,
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    let transcript = sqlx::query!(
        r#"SELECT model, transcript AS "transcript!"
           FROM session_transcripts
           WHERE session_id = $1
           ORDER BY captured_at DESC
           LIMIT 1"#,
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    let Some(transcript_row) = transcript else {
        let summary_row = summary.as_ref();
        let fields = DetailFields {
            user_id: summary_row.map(|s| s.user_id.clone()),
            plugin_id: summary_row.and_then(|s| s.plugin_id.clone()),
            ai_title: summary_row.and_then(|s| s.ai_title.clone()),
            ai_summary: summary_row.and_then(|s| s.ai_summary.clone()),
            model: summary_row.and_then(|s| s.model.clone()),
            started_at: summary_row.and_then(|s| s.started_at),
            ended_at: summary_row.and_then(|s| s.ended_at),
        };
        return Ok(Some(assemble_detail(session_id, fields, vec![])));
    };

    let governance_rows: Vec<GovernanceRow> = sqlx::query_as!(
        GovernanceRow,
        r#"SELECT decision AS "decision!",
                  COALESCE(evaluated_rules, '[]'::jsonb) AS "evaluated_rules!"
           FROM governance_decisions
           WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    let trace_id = sqlx::query!(
        r#"SELECT trace_id AS "trace_id: TraceId" FROM ai_requests
          WHERE session_id = $1 AND trace_id IS NOT NULL
          LIMIT 1"#,
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await?
    .and_then(|r| r.trace_id);

    let turns = parse_turns(&ParseInput {
        session_id: session_id.as_str(),
        transcript: &transcript_row.transcript,
        fallback_model: transcript_row.model.as_deref(),
        governance_rows: &governance_rows,
        fallback_trace_id: trace_id.as_ref().map(TraceId::as_str),
        include_raw,
    });

    let summary_row = summary.as_ref();
    let fields = DetailFields {
        user_id: summary_row.map(|s| s.user_id.clone()),
        plugin_id: summary_row.and_then(|s| s.plugin_id.clone()),
        ai_title: summary_row.and_then(|s| s.ai_title.clone()),
        ai_summary: summary_row.and_then(|s| s.ai_summary.clone()),
        model: summary_row
            .and_then(|s| s.model.clone())
            .or_else(|| transcript_row.model.clone()),
        started_at: summary_row.and_then(|s| s.started_at),
        ended_at: summary_row.and_then(|s| s.ended_at),
    };
    Ok(Some(assemble_detail(session_id, fields, turns)))
}

pub async fn fetch_raw_turns(
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
