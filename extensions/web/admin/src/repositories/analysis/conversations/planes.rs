//! The other planes of one conversation, read beside its fact row.
//!
//! For `/admin/analysis/conversations/{id}`: the turn ledger from the request
//! log, tool calls from `tool_activity` (intent → execution → result, with
//! the artifact rule applied),
//! governance decisions, the skills invoked with the marketplace version
//! served at the time, and the safety scanner's findings.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, ContextId, MarketplaceId, PluginId};

/// One gateway request of the conversation, in order.
#[derive(Debug, Clone)]
pub struct ConversationTurnRow {
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    pub effective_kind: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub requested_model: Option<String>,
    pub route_match: Option<String>,
    pub status: String,
    pub finish_reason: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_tokens: i64,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub upstream_latency_ms: Option<i32>,
    pub is_streaming: bool,
    pub max_tokens: Option<i32>,
    pub tool_calls: i64,
    pub tool_names: Vec<String>,
    pub safety_findings: i64,
    pub safety_blocked: i64,
    pub error_message: Option<String>,
}

pub async fn list_conversation_turns(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationTurnRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT cr.id AS "request_id!", cr.created_at AS "created_at!",
                  cr.effective_kind AS "effective_kind!", cr.provider, cr.model, ar.requested_model,
                  ar.route_match, cr.status AS "status!", ar.finish_reason,
                  COALESCE(cr.input_tokens, 0)::bigint AS "input_tokens!",
                  COALESCE(cr.output_tokens, 0)::bigint AS "output_tokens!",
                  COALESCE(ar.cache_read_tokens, 0)::bigint AS "cache_read_tokens!",
                  COALESCE(ar.cache_creation_tokens, 0)::bigint AS "cache_creation_tokens!",
                  COALESCE(ar.reasoning_tokens, 0)::bigint AS "reasoning_tokens!",
                  cr.cost_microdollars AS "cost_microdollars!", cr.latency_ms, ar.upstream_latency_ms,
                  COALESCE(ar.is_streaming, false) AS "is_streaming!", cr.max_tokens,
                  (SELECT COUNT(*)::bigint FROM ai_request_tool_calls t WHERE t.request_id = cr.id) AS "tool_calls!",
                  ARRAY(SELECT t.tool_name FROM ai_request_tool_calls t WHERE t.request_id = cr.id
                        ORDER BY t.sequence_number) AS "tool_names!",
                  (SELECT COUNT(*)::bigint FROM ai_safety_findings s WHERE s.ai_request_id = cr.id) AS "safety_findings!",
                  (SELECT COUNT(*)::bigint FROM ai_safety_findings s WHERE s.ai_request_id = cr.id AND s.blocked) AS "safety_blocked!",
                  ar.error_message
           FROM conversation_requests cr
           JOIN ai_requests ar ON ar.id = cr.id
           WHERE cr.context_id = $1
           ORDER BY cr.created_at, cr.id
           LIMIT 500"#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationTurnRow {
            request_id: r.request_id,
            created_at: r.created_at,
            effective_kind: r.effective_kind,
            provider: r.provider,
            model: r.model,
            requested_model: r.requested_model,
            route_match: r.route_match,
            status: r.status,
            finish_reason: r.finish_reason,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_read_tokens: r.cache_read_tokens,
            cache_creation_tokens: r.cache_creation_tokens,
            reasoning_tokens: r.reasoning_tokens,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            upstream_latency_ms: r.upstream_latency_ms,
            is_streaming: r.is_streaming,
            max_tokens: r.max_tokens,
            tool_calls: r.tool_calls,
            tool_names: r.tool_names,
            safety_findings: r.safety_findings,
            safety_blocked: r.safety_blocked,
            error_message: r.error_message,
        })
        .collect())
}

/// One tool call from `tool_activity` — intent, execution and result
/// folded, with the one artifact rule's verdict on it.
#[derive(Debug, Clone)]
pub struct ConversationToolCallRow {
    pub tool_name: Option<String>,
    pub server_name: Option<String>,
    pub state: String,
    pub source: Option<String>,
    pub execution_status: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub artifact_id: Option<ArtifactId>,
    pub artifact_type: Option<String>,
    pub artifact_title: Option<String>,
    pub artifact_kind: Option<String>,
    pub input_summary: Option<String>,
    pub is_builtin: bool,
    pub is_structured: bool,
    pub payload_bytes: Option<i32>,
    pub is_error: Option<bool>,
    pub error_message: Option<String>,
    pub request_id: Option<String>,
}

pub async fn list_conversation_tool_calls(
    pool: &PgPool,
    context_id: &ContextId,
    client_session_id: Option<&str>,
) -> Result<Vec<ConversationToolCallRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT t.tool_name, t.server_name, t.state AS "state!", t.source, t.execution_status,
                  t.execution_time_ms::bigint AS execution_time_ms, t.occurred_at,
                  t.artifact_id AS "artifact_id?: ArtifactId", t.artifact_type, t.artifact_title,
                  t.artifact_kind, t.input_summary, COALESCE(t.is_builtin, false) AS "is_builtin!",
                  COALESCE(t.is_structured, false) AS "is_structured!", t.payload_bytes,
                  t.is_error, t.error_message, t.request_id
           FROM tool_activity t
           WHERE t.context_id = $1 OR t.execution_context_id = $1
              OR ($2::text IS NOT NULL AND (t.execution_trace_id = $2 OR t.session_id = $2))
           ORDER BY t.occurred_at NULLS LAST
           LIMIT 500"#,
        context_id.as_str(),
        client_session_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationToolCallRow {
            tool_name: r.tool_name,
            server_name: r.server_name,
            state: r.state,
            source: r.source,
            execution_status: r.execution_status,
            execution_time_ms: r.execution_time_ms,
            occurred_at: r.occurred_at,
            artifact_id: r.artifact_id,
            artifact_type: r.artifact_type,
            artifact_title: r.artifact_title,
            artifact_kind: r.artifact_kind,
            input_summary: r.input_summary,
            is_builtin: r.is_builtin,
            is_structured: r.is_structured,
            payload_bytes: r.payload_bytes,
            is_error: r.is_error,
            error_message: r.error_message,
            request_id: r.request_id,
        })
        .collect())
}

/// One governance decision on a tool call of the conversation.
#[derive(Debug, Clone)]
pub struct ConversationDecisionRow {
    pub tool_name: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub plugin_id: Option<PluginId>,
    pub created_at: DateTime<Utc>,
    // JSON: the chain's per-rule evaluations, shaped by each policy stage.
    pub evaluated_rules: Option<serde_json::Value>,
    pub tool_use_id: Option<String>,
    pub trace_id: Option<String>,
}

pub async fn list_conversation_decisions(
    pool: &PgPool,
    context_id: &ContextId,
    client_session_id: Option<&str>,
) -> Result<Vec<ConversationDecisionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT d.tool_name, d.decision AS "decision!", d.policy, d.reason,
                  d.plugin_id AS "plugin_id?: PluginId",
                  d.created_at AS "created_at!",
                  d.evaluated_rules, d.tool_use_id, d.trace_id
           FROM governance_decisions d
           WHERE d.context_id = $1 OR ($2::text IS NOT NULL AND d.session_id = $2)
           ORDER BY d.created_at
           LIMIT 500"#,
        context_id.as_str(),
        client_session_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationDecisionRow {
            tool_name: r.tool_name,
            decision: r.decision,
            policy: r.policy,
            reason: r.reason,
            plugin_id: r.plugin_id,
            created_at: r.created_at,
            evaluated_rules: r.evaluated_rules,
            tool_use_id: r.tool_use_id,
            trace_id: r.trace_id,
        })
        .collect())
}

/// One skill the conversation invoked and the version served at the time.
#[derive(Debug, Clone)]
pub struct ConversationSkillRow {
    pub plugin_id: Option<PluginId>,
    pub skill: String,
    pub marketplace_id: Option<MarketplaceId>,
    pub marketplace_hash: Option<String>,
    pub invocations: i64,
    pub first_invoked_at: DateTime<Utc>,
}

pub async fn list_conversation_skills(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationSkillRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT s.plugin_id AS "plugin_id?: PluginId", s.skill,
                  s.marketplace_id AS "marketplace_id?: MarketplaceId", s.marketplace_hash,
                  s.invocations, s.first_invoked_at
           FROM conversation_skill_facts s
           WHERE s.context_id = $1
           ORDER BY s.first_invoked_at"#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationSkillRow {
            plugin_id: r.plugin_id,
            skill: r.skill,
            marketplace_id: r.marketplace_id,
            marketplace_hash: r.marketplace_hash,
            invocations: r.invocations,
            first_invoked_at: r.first_invoked_at,
        })
        .collect())
}

/// One safety scanner finding on a request of the conversation.
#[derive(Debug, Clone)]
pub struct ConversationSafetyRow {
    pub phase: String,
    pub category: String,
    pub severity: String,
    pub scanner: String,
    pub blocked: bool,
    pub created_at: DateTime<Utc>,
    pub request_id: String,
    pub excerpt: Option<String>,
}

pub async fn list_conversation_safety_findings(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationSafetyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT s.phase AS "phase!", s.category AS "category!", s.severity AS "severity!",
                  s.scanner, s.blocked AS "blocked!", s.created_at AS "created_at!",
                  s.ai_request_id AS "request_id!", s.excerpt
           FROM ai_safety_findings s
           JOIN conversation_requests cr ON cr.id = s.ai_request_id
           WHERE cr.context_id = $1
           ORDER BY s.created_at
           LIMIT 200"#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationSafetyRow {
            phase: r.phase,
            category: r.category,
            severity: r.severity,
            scanner: r.scanner,
            blocked: r.blocked,
            created_at: r.created_at,
            request_id: r.request_id,
            excerpt: r.excerpt,
        })
        .collect())
}
