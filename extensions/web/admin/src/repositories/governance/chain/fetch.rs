//! Per-table queries for a single `session_id`, each mapping rows into the
//! chain DTOs defined in the parent module.

use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, AiRequestId, PluginId, SessionId, TraceId, UserId};

use super::{AiRequestSummary, ChainUsageEvent, DecisionStage, SessionSummary, TranscriptEnvelope};

#[derive(Debug)]
pub(super) struct DecisionUserSlots {
    pub user_id: UserId,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
}

impl Default for DecisionUserSlots {
    fn default() -> Self {
        Self {
            user_id: UserId::new(String::new()),
            agent_id: None,
            agent_scope: None,
        }
    }
}

pub(super) async fn fetch_decisions(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<(Vec<DecisionStage>, DecisionUserSlots), sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!",
                  user_id as "user_id!: UserId",
                  policy as "policy!",
                  decision as "decision!",
                  reason as "reason!",
                  tool_name as "tool_name!",
                  agent_id as "agent_id: AgentId",
                  agent_scope,
                  plugin_id as "plugin_id: PluginId",
                  COALESCE(evaluated_rules, '[]'::jsonb) as "evaluated_rules!",
                  created_at as "created_at!"
           FROM governance_decisions
           WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    let slots = rows
        .first()
        .map_or_else(DecisionUserSlots::default, |r| DecisionUserSlots {
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

pub(super) async fn fetch_requests(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<(Vec<AiRequestSummary>, Option<TraceId>), sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!",
                  request_id as "request_id!: AiRequestId",
                  trace_id as "trace_id: TraceId",
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
        session_id.as_str(),
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

pub(super) async fn fetch_events(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<ChainUsageEvent>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT id as "id!",
                  event_type as "event_type!",
                  tool_name,
                  plugin_id as "plugin_id: PluginId",
                  description,
                  prompt_preview,
                  COALESCE(metadata, '{}'::jsonb) as "metadata!",
                  created_at as "created_at!"
           FROM plugin_usage_events
           WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| ChainUsageEvent {
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

pub(super) async fn fetch_transcript(
    pool: &PgPool,
    session_id: &SessionId,
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
        session_id.as_str(),
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

pub(super) async fn fetch_summary(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Option<SessionSummary>, sqlx::Error> {
    Ok(sqlx::query!(
        r"SELECT ai_title, ai_summary, ai_tags, model, status, started_at, ended_at
          FROM plugin_session_summaries
          WHERE session_id = $1
          LIMIT 1",
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await?
    .map(|r| SessionSummary {
        ai_title: r.ai_title,
        ai_summary: r.ai_summary,
        ai_tags: r.ai_tags,
        model: r.model,
        status: r.status,
        started_at: r.started_at,
        ended_at: r.ended_at,
    }))
}
