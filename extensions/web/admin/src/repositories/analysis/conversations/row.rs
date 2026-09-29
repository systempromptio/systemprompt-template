//! The conversation row shared by the Analysis pages.
//!
//! It is the `conversation_facts` record plus the judge's columns. The JSON
//! wire form keeps the context id as text; `decode_row` resolves it to the
//! id `SQLx` decoded in the same statement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{ContextId, SessionId, UserId};
use systemprompt_web_shared::{GroupId, ProjectId};

/// One conversation as the pages show it: the fact row, the judge's label
/// and the tokens of its last turns for the row sparkline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationFactRow<Id = ContextId> {
    pub context_id: Id,
    pub title: String,
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub client_session_id: Option<String>,
    pub group_id: Option<GroupId>,
    pub project_id: Option<ProjectId>,
    pub group_name: Option<String>,
    pub project_name: Option<String>,
    pub client_kind: String,
    pub client_attestation: String,
    pub wire_protocol: String,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub models: Vec<String>,
    pub providers: Vec<String>,
    pub request_count: i64,
    pub turn_count: i64,
    pub side_call_count: i64,
    pub side_call_cost_microdollars: i64,
    pub error_count: i64,
    pub rejected_count: i64,
    pub streaming_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
    pub cost_microdollars: i64,
    pub p50_latency_ms: Option<i32>,
    pub p95_latency_ms: Option<i32>,
    pub max_latency_ms: Option<i32>,
    pub tool_calls_intended: i64,
    pub tool_calls_executed: i64,
    pub tool_calls_failed: i64,
    pub artifact_count: i64,
    #[serde(default)]
    pub artifact_files: i64,
    #[serde(default)]
    pub artifact_cards: i64,
    pub safety_findings: i64,
    pub safety_blocked: i64,
    pub gov_allow: i64,
    pub gov_warn: i64,
    pub gov_deny: i64,
    pub prompt_count: i64,
    pub hook_event_count: i64,
    pub hook_status: Option<String>,
    pub skill_invocations: i64,
    pub skills: Vec<String>,
    pub first_at: DateTime<Utc>,
    pub last_at: DateTime<Utc>,
    pub duration_seconds: i64,
    #[serde(default)]
    pub active_ms: i64,
    pub judge_status: Option<String>,
    pub judge_title: Option<String>,
    pub category: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub skills_used: Vec<String>,
    pub outcome: Option<String>,
    pub completion: Option<i16>,
    pub completion_rationale: Option<String>,
    pub confidence: Option<f32>,
    pub classified_at: Option<DateTime<Utc>>,
    pub judge_model: Option<String>,
    pub judge_cost_microdollars: Option<i64>,
    #[serde(default)]
    pub judge_tokens: i64,
    pub judge_trigger: Option<String>,
    #[serde(default)]
    pub turn_tokens: Vec<i64>,
    #[serde(flatten)]
    pub continuation: ContinuationLink,
}

/// A Claude Code session resumed after compaction, and the conversation it
/// most likely continues (text, since it only builds a link).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContinuationLink {
    #[serde(default)]
    pub is_continuation: bool,
    #[serde(default)]
    pub prev_context_id: Option<String>,
    #[serde(default)]
    pub prev_title: Option<String>,
}

pub(super) fn decode_row(
    r: ConversationFactRow<String>,
    ids: &[ContextId],
) -> Result<ConversationFactRow, sqlx::Error> {
    let context_id = crate::repositories::dashboard_read::context_id(&r.context_id, ids)?;
    Ok(ConversationFactRow {
        context_id,
        title: r.title,
        user_id: r.user_id,
        display_name: r.display_name,
        session_id: r.session_id,
        client_session_id: r.client_session_id,
        group_id: r.group_id,
        project_id: r.project_id,
        group_name: r.group_name,
        project_name: r.project_name,
        client_kind: r.client_kind,
        client_attestation: r.client_attestation,
        wire_protocol: r.wire_protocol,
        model: r.model,
        provider: r.provider,
        models: r.models,
        providers: r.providers,
        request_count: r.request_count,
        turn_count: r.turn_count,
        side_call_count: r.side_call_count,
        side_call_cost_microdollars: r.side_call_cost_microdollars,
        error_count: r.error_count,
        rejected_count: r.rejected_count,
        streaming_count: r.streaming_count,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        cache_read_tokens: r.cache_read_tokens,
        cache_creation_tokens: r.cache_creation_tokens,
        reasoning_tokens: r.reasoning_tokens,
        total_tokens: r.total_tokens,
        cost_microdollars: r.cost_microdollars,
        p50_latency_ms: r.p50_latency_ms,
        p95_latency_ms: r.p95_latency_ms,
        max_latency_ms: r.max_latency_ms,
        tool_calls_intended: r.tool_calls_intended,
        tool_calls_executed: r.tool_calls_executed,
        tool_calls_failed: r.tool_calls_failed,
        artifact_count: r.artifact_count,
        artifact_files: r.artifact_files,
        artifact_cards: r.artifact_cards,
        safety_findings: r.safety_findings,
        safety_blocked: r.safety_blocked,
        gov_allow: r.gov_allow,
        gov_warn: r.gov_warn,
        gov_deny: r.gov_deny,
        prompt_count: r.prompt_count,
        hook_event_count: r.hook_event_count,
        hook_status: r.hook_status,
        skill_invocations: r.skill_invocations,
        skills: r.skills,
        first_at: r.first_at,
        last_at: r.last_at,
        duration_seconds: r.duration_seconds,
        active_ms: r.active_ms,
        judge_status: r.judge_status,
        judge_title: r.judge_title,
        category: r.category,
        summary: r.summary,
        tags: r.tags,
        skills_used: r.skills_used,
        outcome: r.outcome,
        completion: r.completion,
        completion_rationale: r.completion_rationale,
        confidence: r.confidence,
        classified_at: r.classified_at,
        judge_model: r.judge_model,
        judge_cost_microdollars: r.judge_cost_microdollars,
        judge_tokens: r.judge_tokens,
        judge_trigger: r.judge_trigger,
        turn_tokens: r.turn_tokens,
        continuation: r.continuation,
    })
}
