//! Decision chain assembly.
//!
//! `governance_decisions` and `plugin_usage_events` do not carry a `trace_id`
//! column today, so the chain is anchored on `session_id` (shared by all four
//! tables) and surfaces the `trace_id` from `ai_requests` when available.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize)]
pub struct ChainIdentity {
    pub user_id: String,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionStage {
    pub id: String,
    pub policy: String,
    pub decision: String,
    pub reason: String,
    pub tool_name: String,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub plugin_id: Option<String>,
    pub evaluated_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiRequestSummary {
    pub id: String,
    pub request_id: String,
    pub trace_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageEvent {
    pub id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub description: Option<String>,
    pub prompt_preview: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptEnvelope {
    pub id: String,
    pub model: Option<String>,
    pub entries_counted: Option<i32>,
    pub total_input_tokens: Option<i64>,
    pub total_output_tokens: Option<i64>,
    pub captured_at: DateTime<Utc>,
    pub transcript: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionSummary {
    pub ai_title: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_tags: Option<String>,
    pub model: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ChainTotals {
    pub decision_count: i64,
    pub deny_count: i64,
    pub event_count: i64,
    pub request_count: i64,
    pub total_cost_microdollars: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainEnvelope {
    pub trace_id: Option<String>,
    pub session_id: String,
    pub identity: ChainIdentity,
    pub decisions: Vec<DecisionStage>,
    pub requests: Vec<AiRequestSummary>,
    pub events: Vec<UsageEvent>,
    pub transcript: Option<TranscriptEnvelope>,
    pub summary: Option<SessionSummary>,
    pub totals: ChainTotals,
}

/// Resolve `id` (`decision_id`, `request_id`, `trace_id`, or `session_id`) to a `session_id`.
async fn resolve_session_id(pool: &PgPool, id: &str) -> Result<Option<String>, sqlx::Error> {
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id!" FROM governance_decisions WHERE id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r"SELECT session_id FROM ai_requests
          WHERE id = $1 OR request_id = $1 OR trace_id = $1
          LIMIT 1",
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        if let Some(sid) = row.session_id {
            return Ok(Some(sid));
        }
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id!" FROM plugin_usage_events WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    Ok(None)
}

#[derive(Debug, Default)]
struct DecisionUserSlots {
    user_id: String,
    agent_id: Option<String>,
    agent_scope: Option<String>,
}

async fn fetch_decisions(
    pool: &PgPool,
    session_id: &str,
) -> Result<(Vec<DecisionStage>, DecisionUserSlots), sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!",
                  user_id as "user_id!",
                  policy as "policy!",
                  decision as "decision!",
                  reason as "reason!",
                  tool_name as "tool_name!",
                  agent_id,
                  agent_scope,
                  plugin_id,
                  COALESCE(evaluated_rules, '[]'::jsonb) as "evaluated_rules!",
                  created_at as "created_at!"
           FROM governance_decisions
           WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    let slots = rows.first().map_or_else(DecisionUserSlots::default, |r| DecisionUserSlots {
        user_id: r.user_id.clone(),
        agent_id: r.agent_id.clone(),
        agent_scope: r.agent_scope.clone(),
    });

    let stages = rows
        .into_iter()
        .map(|r| DecisionStage {
            id: r.id,
            policy: r.policy,
            decision: r.decision,
            reason: r.reason,
            tool_name: r.tool_name,
            agent_id: r.agent_id,
            agent_scope: r.agent_scope,
            plugin_id: r.plugin_id,
            evaluated_rules: r.evaluated_rules,
            created_at: r.created_at,
        })
        .collect();

    Ok((stages, slots))
}

async fn fetch_requests(
    pool: &PgPool,
    session_id: &str,
) -> Result<(Vec<AiRequestSummary>, Option<String>), sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!",
                  request_id as "request_id!",
                  trace_id,
                  provider as "provider!",
                  model as "model!",
                  status as "status!",
                  input_tokens,
                  output_tokens,
                  COALESCE(cost_microdollars, 0)::bigint as "cost_microdollars!",
                  latency_ms,
                  error_message,
                  created_at as "created_at!"
           FROM ai_requests
           WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?;

    let trace_id = rows.iter().find_map(|r| r.trace_id.clone());

    let requests = rows
        .into_iter()
        .map(|r| AiRequestSummary {
            id: r.id,
            request_id: r.request_id,
            trace_id: r.trace_id,
            provider: r.provider,
            model: r.model,
            status: r.status,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            error_message: r.error_message,
            created_at: r.created_at,
        })
        .collect();

    Ok((requests, trace_id))
}

async fn fetch_events(pool: &PgPool, session_id: &str) -> Result<Vec<UsageEvent>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT id as "id!",
                  event_type as "event_type!",
                  tool_name,
                  plugin_id,
                  description,
                  prompt_preview,
                  COALESCE(metadata, '{}'::jsonb) as "metadata!",
                  created_at as "created_at!"
           FROM plugin_usage_events
           WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| UsageEvent {
        id: r.id,
        event_type: r.event_type,
        tool_name: r.tool_name,
        plugin_id: r.plugin_id,
        description: r.description,
        prompt_preview: r.prompt_preview,
        metadata: r.metadata,
        created_at: r.created_at,
    })
    .collect())
}

async fn fetch_transcript(
    pool: &PgPool,
    session_id: &str,
) -> Result<Option<TranscriptEnvelope>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT id as "id!",
                  model,
                  entries_counted,
                  total_input_tokens,
                  total_output_tokens,
                  captured_at as "captured_at!",
                  transcript as "transcript!"
           FROM session_transcripts
           WHERE session_id = $1
           ORDER BY captured_at DESC
           LIMIT 1"#,
        session_id,
    )
    .fetch_optional(pool)
    .await?
    .map(|r| TranscriptEnvelope {
        id: r.id,
        model: r.model,
        entries_counted: r.entries_counted,
        total_input_tokens: r.total_input_tokens,
        total_output_tokens: r.total_output_tokens,
        captured_at: r.captured_at,
        transcript: r.transcript,
    }))
}

async fn fetch_summary(
    pool: &PgPool,
    session_id: &str,
) -> Result<Option<SessionSummary>, sqlx::Error> {
    Ok(sqlx::query!(
        r"SELECT ai_title, ai_summary, ai_tags, model, status, started_at, ended_at
          FROM plugin_session_summaries
          WHERE session_id = $1
          LIMIT 1",
        session_id,
    )
    .fetch_optional(pool)
    .await?
    .map(|r| SessionSummary {
        ai_title: r.ai_title,
        ai_summary: r.ai_summary,
        ai_tags: r.ai_tags,
        model: r.model,
        status: Some(r.status),
        started_at: r.started_at,
        ended_at: r.ended_at,
    }))
}

fn compute_totals(
    decisions: &[DecisionStage],
    requests: &[AiRequestSummary],
    events: &[UsageEvent],
) -> ChainTotals {
    let mut totals = ChainTotals {
        decision_count: decisions.len() as i64,
        deny_count: decisions.iter().filter(|d| d.decision == "deny").count() as i64,
        event_count: events.len() as i64,
        request_count: requests.len() as i64,
        ..ChainTotals::default()
    };
    for r in requests {
        totals.total_cost_microdollars += r.cost_microdollars;
        totals.total_input_tokens += i64::from(r.input_tokens.unwrap_or(0));
        totals.total_output_tokens += i64::from(r.output_tokens.unwrap_or(0));
    }
    totals
}

/// Fetch the full decision chain for an identifier.
///
/// `id` may be a `decision_id`, `request_id`, `trace_id`, or `session_id`.
/// Returns `Ok(None)` if the id does not resolve to any session.
pub async fn fetch_decision_chain(
    pool: &PgPool,
    id: &str,
) -> Result<Option<ChainEnvelope>, sqlx::Error> {
    let Some(session_id) = resolve_session_id(pool, id).await? else {
        return Ok(None);
    };

    let (decisions, slots) = fetch_decisions(pool, &session_id).await?;
    let (requests, trace_id) = fetch_requests(pool, &session_id).await?;
    let events = fetch_events(pool, &session_id).await?;
    let transcript = fetch_transcript(pool, &session_id).await?;
    let summary = fetch_summary(pool, &session_id).await?;

    let identity = ChainIdentity {
        user_id: slots.user_id,
        agent_id: slots.agent_id,
        agent_scope: slots.agent_scope,
    };

    let totals = compute_totals(&decisions, &requests, &events);

    Ok(Some(ChainEnvelope {
        trace_id,
        session_id,
        identity,
        decisions,
        requests,
        events,
        transcript,
        summary,
        totals,
    }))
}
