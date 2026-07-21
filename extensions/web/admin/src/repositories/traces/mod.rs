//! Trace explorer queries.
//!
//! A "trace" here is keyed on `session_id` (which all four governance / gateway
//! / events tables share) and surfaces the `trace_id` from `ai_requests` when
//! one exists. [`list_traces`] returns one summary row per session in the
//! window; [`list_trace_spans`] returns the union of per-table rows for a
//! single session, normalised into a [`Span`] shape and ordered by start time.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{AgentId, SessionId, TraceId, UserId};

mod list;
mod spans;
mod stats;

pub use list::{TracePage, list_traces};
pub use spans::{list_trace_spans, resolve_trace_session};
pub use stats::get_trace_stats;

#[derive(Debug, Clone, Serialize)]
pub struct TraceSummary {
    pub session_id: SessionId,
    pub trace_id: Option<TraceId>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub user_id: Option<UserId>,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub span_count: i64,
    pub request_count: i64,
    pub tool_call_count: i64,
    pub governance_count: i64,
    pub deny_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub total_latency_ms: i64,
    pub cache_hit_any: bool,
    pub top_tool: Option<String>,
    pub has_error: bool,
    pub has_deny: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TraceFilter<'a> {
    pub user_id: Option<&'a str>,
    pub agent_id: Option<&'a str>,
    pub agent_scope: Option<&'a str>,
    pub policy: Option<&'a str>,
    pub decision: Option<&'a str>,
    pub error_only: bool,
    pub deny_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TraceSortColumn {
    StartedAt,
    Duration,
    SpanCount,
    Cost,
    Tokens,
}

impl TraceSortColumn {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::StartedAt => "started_at",
            Self::Duration => "duration",
            Self::SpanCount => "span_count",
            Self::Cost => "cost",
            Self::Tokens => "tokens",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TraceSortDir {
    Asc,
    Desc,
}

impl TraceSortDir {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TraceSort {
    pub column: TraceSortColumn,
    pub dir: TraceSortDir,
}

impl Default for TraceSort {
    fn default() -> Self {
        Self {
            column: TraceSortColumn::StartedAt,
            dir: TraceSortDir::Desc,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TraceStats {
    pub total_traces: i64,
    pub error_count: i64,
    pub deny_count: i64,
    pub p50_duration_ms: i64,
    pub p95_duration_ms: i64,
    pub p99_duration_ms: i64,
}

/// One waterfall span — normalised across the four span sources.
#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub id: String,
    pub kind: SpanKind,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_ms: i64,
    pub status: SpanStatus,
    pub identity_label: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    Gateway,
    Governance,
    Tool,
    Model,
    Spawn,
}

impl SpanKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Governance => "governance",
            Self::Tool => "tool",
            Self::Model => "model",
            Self::Spawn => "spawn",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    Ok,
    Deny,
    Error,
    Pending,
}

impl SpanStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Deny => "deny",
            Self::Error => "error",
            Self::Pending => "pending",
        }
    }
}
