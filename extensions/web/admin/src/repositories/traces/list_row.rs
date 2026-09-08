//! The raw `list_traces` result row and its lift into [`TraceSummary`].
//!
//! Split from `list.rs` so the query module holds the SQL and nothing else.

use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};

use super::TraceSummary;

#[derive(Debug)]
pub(super) struct TraceListRow {
    pub(super) session_id: SessionId,
    pub(super) trace_id: Option<TraceId>,
    pub(super) started_at: chrono::DateTime<chrono::Utc>,
    pub(super) ended_at: chrono::DateTime<chrono::Utc>,
    pub(super) active_ms: i64,
    pub(super) window_ms: i64,
    pub(super) user_id: Option<UserId>,
    pub(super) user_label: Option<String>,
    pub(super) agent_id: Option<AgentId>,
    pub(super) agent_scope: Option<String>,
    pub(super) model: Option<String>,
    pub(super) provider: Option<String>,
    pub(super) span_count: i64,
    pub(super) request_count: i64,
    pub(super) tool_call_count: i64,
    pub(super) governance_count: i64,
    pub(super) deny_count: i64,
    pub(super) total_tokens: i64,
    pub(super) input_tokens: i64,
    pub(super) output_tokens: i64,
    pub(super) total_cost_microdollars: i64,
    pub(super) total_latency_ms: i64,
    pub(super) cache_hit_any: bool,
    pub(super) top_tool: Option<String>,
    pub(super) has_error: bool,
    pub(super) error_count: i64,
    pub(super) has_deny: bool,
    pub(super) total_count: i64,
}

impl From<TraceListRow> for TraceSummary {
    fn from(r: TraceListRow) -> Self {
        Self {
            session_id: r.session_id,
            trace_id: r.trace_id,
            started_at: r.started_at,
            ended_at: r.ended_at,
            active_ms: r.active_ms,
            window_ms: r.window_ms,
            user_id: r.user_id,
            user_label: r.user_label,
            agent_id: r.agent_id,
            agent_scope: r.agent_scope,
            model: r.model,
            provider: r.provider,
            span_count: r.span_count,
            request_count: r.request_count,
            tool_call_count: r.tool_call_count,
            governance_count: r.governance_count,
            deny_count: r.deny_count,
            total_tokens: r.total_tokens,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            total_cost_microdollars: r.total_cost_microdollars,
            total_latency_ms: r.total_latency_ms,
            cache_hit_any: r.cache_hit_any,
            top_tool: r.top_tool,
            has_error: r.has_error,
            error_count: r.error_count,
            has_deny: r.has_deny,
        }
    }
}
