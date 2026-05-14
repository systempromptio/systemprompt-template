//! Conversations & Transcripts page data layer.
//!
//! `fetch_conversation_list` powers the left pane (sessions filtered by
//! time-range / identity / free-text). `fetch_conversation_detail` parses the
//! JSONB `session_transcripts.transcript` into a flat `Vec<TranscriptTurn>`,
//! enriches each turn with any matching `governance_decisions` row, and
//! exposes both a redacted (default) and an optional raw text body.
//!
//! Free-text search relies on the `idx_session_transcripts_jsonb` GIN index
//! (`jsonb_path_ops`). Pure substring searches use `ILIKE` against the JSONB
//! cast to text — that is unindexed and capped at 200 rows by the SQL filter
//! upstream so the cost is bounded.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

/// One row in the left-pane session list.
#[derive(Debug, Clone, Serialize)]
pub struct ConversationListItem {
    pub session_id: String,
    pub user_id: String,
    pub plugin_id: Option<String>,
    pub model: Option<String>,
    pub status: Option<String>,
    pub ai_title: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub governance_intervention_count: i64,
    pub deny_count: i64,
}

/// Filter inputs for the left-pane query (all optional).
#[derive(Debug, Clone, Default)]
pub struct ConversationListFilter {
    pub user_id: Option<String>,
    pub plugin_id: Option<String>,
    pub free_text: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: i64,
}

/// One enriched turn for the right-pane render.
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptTurn {
    pub id: String,
    pub session_id: String,
    pub ordinal: i32,
    pub role: String,
    pub ts: Option<DateTime<Utc>>,
    pub model: Option<String>,
    pub latency_ms: Option<i32>,
    /// Always populated. PII-bearing substrings are replaced with sentinels.
    pub content_redacted: Option<String>,
    pub redactions_applied: u32,
    /// Only populated when the caller holds `transcript:view_pii`.
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub governance: Option<TurnGovernance>,
    pub anomaly_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCall {
    pub id: Option<String>,
    pub name: String,
    pub args_json: serde_json::Value,
    pub result_json: Option<serde_json::Value>,
    pub duration_ms: Option<i32>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnGovernance {
    pub decision: String,
    pub trace_id: Option<String>,
    pub rule_count: i32,
    pub redactions_applied: u32,
}

/// Top-level detail returned by `fetch_conversation_detail`.
#[derive(Debug, Clone, Serialize)]
pub struct ConversationDetail {
    pub session_id: String,
    pub user_id: Option<String>,
    pub plugin_id: Option<String>,
    pub ai_title: Option<String>,
    pub ai_summary: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub turns: Vec<TranscriptTurn>,
}

pub async fn fetch_conversation_list(
    pool: &PgPool,
    filter: &ConversationListFilter,
) -> Result<Vec<ConversationListItem>, sqlx::Error> {
    let limit = if filter.limit > 0 && filter.limit <= 500 {
        filter.limit
    } else {
        100
    };
    let free_text_pattern = filter
        .free_text
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")));

    let rows = sqlx::query!(
        r#"
        SELECT s.session_id    AS "session_id!",
               s.user_id       AS "user_id!",
               s.plugin_id,
               s.model,
               s.status,
               s.ai_title,
               s.started_at,
               COALESCE(s.total_input_tokens, 0)::bigint  AS "total_input_tokens!",
               COALESCE(s.total_output_tokens, 0)::bigint AS "total_output_tokens!",
               COALESCE(g.intervention_count, 0)::bigint  AS "governance_intervention_count!",
               COALESCE(g.deny_count, 0)::bigint          AS "deny_count!"
        FROM plugin_session_summaries s
        LEFT JOIN (
            SELECT session_id,
                   COUNT(*)                                            AS intervention_count,
                   COUNT(*) FILTER (WHERE decision = 'deny')           AS deny_count
            FROM governance_decisions
            GROUP BY session_id
        ) g ON g.session_id = s.session_id
        LEFT JOIN session_transcripts t ON t.session_id = s.session_id
        WHERE ($1::text IS NULL OR s.user_id   = $1)
          AND ($2::text IS NULL OR s.plugin_id = $2)
          AND ($3::timestamptz IS NULL OR s.started_at >= $3)
          AND ($4::timestamptz IS NULL OR s.started_at <  $4)
          AND ($5::text IS NULL
               OR s.ai_title ILIKE $5
               OR s.ai_summary ILIKE $5
               OR t.transcript::text ILIKE $5)
        ORDER BY s.started_at DESC NULLS LAST
        LIMIT $6
        "#,
        filter.user_id,
        filter.plugin_id,
        filter.since,
        filter.until,
        free_text_pattern,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ConversationListItem {
            session_id: r.session_id,
            user_id: r.user_id,
            plugin_id: r.plugin_id,
            model: r.model,
            status: Some(r.status),
            ai_title: r.ai_title,
            started_at: r.started_at,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            governance_intervention_count: r.governance_intervention_count,
            deny_count: r.deny_count,
        })
        .collect())
}

pub async fn fetch_conversation_detail(
    pool: &PgPool,
    session_id: &str,
    include_raw: bool,
) -> Result<Option<ConversationDetail>, sqlx::Error> {
    let summary = sqlx::query!(
        r"SELECT user_id, plugin_id, ai_title, ai_summary, model, started_at, ended_at
          FROM plugin_session_summaries
          WHERE session_id = $1
          LIMIT 1",
        session_id,
    )
    .fetch_optional(pool)
    .await?;

    let transcript = sqlx::query!(
        r#"SELECT model, transcript AS "transcript!"
           FROM session_transcripts
           WHERE session_id = $1
           ORDER BY captured_at DESC
           LIMIT 1"#,
        session_id,
    )
    .fetch_optional(pool)
    .await?;

    let Some(transcript_row) = transcript else {
        // No transcript captured yet — return empty turns so the page still renders.
        let summary_row = summary.as_ref();
        return Ok(Some(ConversationDetail {
            session_id: session_id.to_string(),
            user_id: summary_row.map(|s| s.user_id.clone()),
            plugin_id: summary_row.and_then(|s| s.plugin_id.clone()),
            ai_title: summary_row.and_then(|s| s.ai_title.clone()),
            ai_summary: summary_row.and_then(|s| s.ai_summary.clone()),
            model: summary_row.and_then(|s| s.model.clone()),
            started_at: summary_row.and_then(|s| s.started_at),
            ended_at: summary_row.and_then(|s| s.ended_at),
            turns: vec![],
        }));
    };

    let governance_rows: Vec<GovernanceRow> = sqlx::query_as!(
        GovernanceRow,
        r#"SELECT decision AS "decision!",
                  COALESCE(evaluated_rules, '[]'::jsonb) AS "evaluated_rules!"
           FROM governance_decisions
           WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    let trace_id = sqlx::query!(
        r"SELECT trace_id FROM ai_requests
          WHERE session_id = $1 AND trace_id IS NOT NULL
          LIMIT 1",
        session_id,
    )
    .fetch_optional(pool)
    .await?
    .and_then(|r| r.trace_id);

    let turns = parse_turns(&ParseInput {
        session_id,
        transcript: &transcript_row.transcript,
        fallback_model: transcript_row.model.as_deref(),
        governance_rows: &governance_rows,
        fallback_trace_id: trace_id.as_deref(),
        include_raw,
    });

    let summary_row = summary.as_ref();
    Ok(Some(ConversationDetail {
        session_id: session_id.to_string(),
        user_id: summary_row.map(|s| s.user_id.clone()),
        plugin_id: summary_row.and_then(|s| s.plugin_id.clone()),
        ai_title: summary_row.and_then(|s| s.ai_title.clone()),
        ai_summary: summary_row.and_then(|s| s.ai_summary.clone()),
        model: summary_row
            .and_then(|s| s.model.clone())
            .or_else(|| transcript_row.model.clone()),
        started_at: summary_row.and_then(|s| s.started_at),
        ended_at: summary_row.and_then(|s| s.ended_at),
        turns,
    }))
}

/// Just the raw turn bodies, keyed by ordinal — the capability-gated endpoint
/// returns this when the viewer holds `transcript:view_pii`.
#[derive(Debug, Clone, Serialize)]
pub struct RawTurnBody {
    pub ordinal: i32,
    pub content: String,
}

pub async fn fetch_raw_turns(
    pool: &PgPool,
    session_id: &str,
) -> Result<Option<Vec<RawTurnBody>>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT transcript AS "transcript!"
           FROM session_transcripts
           WHERE session_id = $1
           ORDER BY captured_at DESC
           LIMIT 1"#,
        session_id,
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

// ── Internals ──────────────────────────────────────────────────────────────

struct ParseInput<'a> {
    session_id: &'a str,
    transcript: &'a serde_json::Value,
    fallback_model: Option<&'a str>,
    governance_rows: &'a [GovernanceRow],
    fallback_trace_id: Option<&'a str>,
    include_raw: bool,
}

fn parse_turns(input: &ParseInput<'_>) -> Vec<TranscriptTurn> {
    let Some(arr) = input.transcript.as_array() else {
        return vec![];
    };

    arr.iter()
        .enumerate()
        .map(|(idx, entry)| {
            let ordinal = i32::try_from(idx).unwrap_or(i32::MAX);
            let role = entry
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("assistant")
                .to_string();
            let ts = entry
                .get("ts")
                .or_else(|| entry.get("timestamp"))
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc));
            let model = entry
                .get("model")
                .and_then(|v| v.as_str())
                .map(String::from)
                .or_else(|| input.fallback_model.map(String::from));
            let latency_ms = entry
                .get("latency_ms")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok());

            let raw_text = extract_content_text(entry);
            let (redacted_text, redactions_applied) = redact_text(&raw_text);
            let content_redacted = Some(redacted_text);
            let content = if input.include_raw {
                Some(raw_text)
            } else {
                None
            };

            let tool_calls = entry
                .get("tool_calls")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().map(parse_tool_call).collect())
                .unwrap_or_default();

            let governance =
                match_governance(input.governance_rows, ordinal, input.fallback_trace_id);
            let anomaly_count = entry
                .get("anomaly_count")
                .and_then(serde_json::Value::as_i64)
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or(0);

            TranscriptTurn {
                id: format!("{}:{ordinal}", input.session_id),
                session_id: input.session_id.to_string(),
                ordinal,
                role,
                ts,
                model,
                latency_ms,
                content_redacted,
                redactions_applied,
                content,
                tool_calls,
                governance,
                anomaly_count,
            }
        })
        .collect()
}

fn parse_tool_call(v: &serde_json::Value) -> ToolCall {
    ToolCall {
        id: v.get("id").and_then(|x| x.as_str()).map(String::from),
        name: v
            .get("name")
            .or_else(|| v.get("tool_name"))
            .and_then(|x| x.as_str())
            .unwrap_or("unknown")
            .to_string(),
        args_json: v
            .get("args")
            .or_else(|| v.get("input"))
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        result_json: v.get("result").or_else(|| v.get("output")).cloned(),
        duration_ms: v
            .get("duration_ms")
            .and_then(serde_json::Value::as_i64)
            .and_then(|d| i32::try_from(d).ok()),
        status: v
            .get("status")
            .and_then(|x| x.as_str())
            .unwrap_or("ok")
            .to_string(),
    }
}

/// Pull a textual representation of a transcript entry's body.
/// Accepts plain strings, Anthropic-style content arrays, or `text` fields.
fn extract_content_text(entry: &serde_json::Value) -> String {
    if let Some(s) = entry.get("content").and_then(|v| v.as_str()) {
        return s.to_string();
    }
    if let Some(arr) = entry.get("content").and_then(|v| v.as_array()) {
        let mut out = String::new();
        for block in arr {
            if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(t);
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    if let Some(s) = entry.get("text").and_then(|v| v.as_str()) {
        return s.to_string();
    }
    String::new()
}

/// Defense-in-depth text redactor for prompts/responses heading to the DOM.
///
/// Catches the common high-entropy / well-prefixed credential shapes; not a
/// substitute for the structural `secret_scan` policy run at webhook time.
/// Returns `(redacted_text, count_of_redactions)`.
pub fn redact_text(input: &str) -> (String, u32) {
    const PREFIX_PATTERNS: &[(&str, &str)] = &[
        ("AKIA", "aws_access_key"),
        ("ASIA", "aws_session_key"),
        ("ghp_", "github_token"),
        ("github_pat_", "github_token"),
        ("gho_", "github_oauth"),
        ("ghu_", "github_user_token"),
        ("ghs_", "github_server_token"),
        ("ghr_", "github_refresh"),
        ("glpat-", "gitlab_token"),
        ("xoxb-", "slack_bot_token"),
        ("xoxp-", "slack_user_token"),
        ("sk-ant-", "anthropic_api_key"),
        ("sk-proj-", "openai_api_key"),
        ("sk_live_", "stripe_secret_key"),
        ("rk_live_", "stripe_restricted_key"),
        ("AIza", "google_api_key"),
        ("SG.", "sendgrid_api_key"),
    ];

    let mut out = String::with_capacity(input.len());
    let mut count: u32 = 0;
    let mut idx = 0usize;
    let bytes = input.as_bytes();
    while idx < bytes.len() {
        let mut hit: Option<(usize, &str)> = None;
        for &(prefix, label) in PREFIX_PATTERNS {
            if input[idx..].starts_with(prefix) {
                hit = Some((prefix.len(), label));
                break;
            }
        }
        if let Some((prefix_len, label)) = hit {
            // consume the prefix plus any following non-whitespace, non-quote chars
            let mut end = idx + prefix_len;
            while end < bytes.len() {
                let b = bytes[end];
                if b.is_ascii_whitespace() || b == b'"' || b == b'\'' || b == b',' || b == b')' {
                    break;
                }
                end += 1;
            }
            out.push_str(&format!("[REDACTED:{label}]"));
            count = count.saturating_add(1);
            idx = end;
        } else {
            let ch = input[idx..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&input[idx..idx + ch]);
            idx += ch;
        }
    }
    (out, count)
}

struct GovernanceRow {
    decision: String,
    evaluated_rules: serde_json::Value,
}

fn match_governance(
    rows: &[GovernanceRow],
    _ordinal: i32,
    fallback_trace_id: Option<&str>,
) -> Option<TurnGovernance> {
    // No per-turn binding column on `governance_decisions` today, so we fall
    // back to a session-wide pick: prefer the first deny, else the first row.
    let row = rows
        .iter()
        .find(|r| r.decision == "deny")
        .or_else(|| rows.first())?;

    let rule_count = i32::try_from(row.evaluated_rules.as_array().map_or(0, Vec::len)).unwrap_or(0);

    Some(TurnGovernance {
        decision: row.decision.clone(),
        trace_id: fallback_trace_id.map(String::from),
        rule_count,
        redactions_applied: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_aws_key() {
        let (out, n) = redact_text("here is AKIAIOSFODNN7EXAMPLE in text");
        assert_eq!(n, 1);
        assert!(out.contains("[REDACTED:aws_access_key]"));
        assert!(!out.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn redact_anthropic_key() {
        let (out, n) = redact_text("call sk-ant-api03-abc and also AIzaSyAbCdEfG please");
        assert_eq!(n, 2);
        assert!(out.contains("[REDACTED:anthropic_api_key]"));
        assert!(out.contains("[REDACTED:google_api_key]"));
    }

    #[test]
    fn redact_no_op_on_clean_text() {
        let (out, n) = redact_text("hello world, no secrets here");
        assert_eq!(n, 0);
        assert_eq!(out, "hello world, no secrets here");
    }
}
