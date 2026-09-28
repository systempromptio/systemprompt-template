//! `/admin/traces` — one row per trace in the window.

use async_trait::async_trait;

use super::requests::time_range;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_perf_traces::export::export_rows;
use crate::repositories::traces::TraceSummary;
use systemprompt::identifiers::{AgentId, TraceId, UserId};

pub(crate) struct Traces;

const COLUMNS: &[Column] = &[
    Column::new("started_at", "Started", CellKind::Timestamp),
    Column::new("ended_at", "Ended", CellKind::Timestamp),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("trace_id", "Trace", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("user_label", "Name", CellKind::Text).optional(),
    Column::new("agent_id", "Agent", CellKind::Text).optional(),
    Column::new("agent_scope", "Scope", CellKind::Text).optional(),
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("active_ms", "Active (ms)", CellKind::Integer),
    Column::new("window_ms", "Window (ms)", CellKind::Integer).optional(),
    Column::new("spans", "Spans", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("tool_calls", "Tool calls", CellKind::Integer),
    Column::new("governance", "Governance evaluations", CellKind::Integer),
    Column::new("denies", "Denies", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("latency_ms", "Total latency (ms)", CellKind::Integer).optional(),
    Column::new("cache_hit", "Cache hit", CellKind::Bool).optional(),
    Column::new("top_tool", "Top tool", CellKind::Text).optional(),
    Column::new("errors", "Errors", CellKind::Integer),
];

fn row(r: &TraceSummary) -> Vec<Cell> {
    vec![
        r.started_at.into(),
        r.ended_at.into(),
        r.session_id.as_str().into(),
        Cell::opt_text(r.trace_id.as_ref().map(TraceId::as_str)),
        Cell::opt_text(r.user_id.as_ref().map(UserId::as_str)),
        Cell::opt_text(r.user_label.as_deref()),
        Cell::opt_text(r.agent_id.as_ref().map(AgentId::as_str)),
        Cell::opt_text(r.agent_scope.as_deref()),
        Cell::opt_text(r.provider.as_deref()),
        Cell::opt_text(r.model.as_deref()),
        r.active_ms.into(),
        r.window_ms.into(),
        r.span_count.into(),
        r.request_count.into(),
        r.tool_call_count.into(),
        r.governance_count.into(),
        r.deny_count.into(),
        r.input_tokens.into(),
        r.output_tokens.into(),
        Cell::Money(r.total_cost_microdollars),
        r.total_latency_ms.into(),
        r.cache_hit_any.into(),
        Cell::opt_text(r.top_tool.as_deref()),
        r.error_count.into(),
    ]
}

#[async_trait]
impl DataSet for Traces {
    fn id(&self) -> &'static str {
        "traces"
    }
    fn title(&self) -> &'static str {
        "Traces"
    }
    fn description(&self) -> &'static str {
        "One row per trace in the window: who, model, spans, requests, tool calls, tokens and cost."
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (rows, total) = export_rows(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
        )
        .await?;
        Ok(Table {
            rows: rows.iter().map(row).collect(),
            total,
        })
    }
}
