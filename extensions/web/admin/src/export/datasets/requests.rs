//! `/admin/requests` — one row per `/v1/messages` call.

use async_trait::async_trait;
use systemprompt_web_shared::{GroupId, ProjectId};

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_analytics_requests::export_rows;
use crate::repositories::analytics::requests::RequestRow;
use crate::util::time_range::{TimeRange, TimeRangePreset};
use systemprompt::identifiers::{SessionId, TraceId};

pub(crate) struct Requests;

const COLUMNS: &[Column] = &[
    Column::new("created_at", "Time", CellKind::Timestamp),
    Column::new("request_id", "Request", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("user_label", "Name", CellKind::Text).optional(),
    Column::new("group_id", "Group", CellKind::Text),
    Column::new("group_name", "Group name", CellKind::Text).optional(),
    Column::new("project_id", "Project", CellKind::Text),
    Column::new("project_name", "Project name", CellKind::Text).optional(),
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("status", "Status", CellKind::Text),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("latency_ms", "Latency (ms)", CellKind::Integer),
    Column::new("tool_calls", "Tool calls", CellKind::Integer),
    Column::new("decisions", "Governance decisions", CellKind::Integer).optional(),
    Column::new("deny_count", "Denies", CellKind::Integer),
    Column::new("session_id", "Session", CellKind::Text).optional(),
    Column::new("trace_id", "Trace", CellKind::Text).optional(),
    Column::new("error", "Error", CellKind::Text).optional(),
];

fn row(r: &RequestRow) -> Vec<Cell> {
    vec![
        r.created_at.into(),
        r.request_id.as_str().into(),
        r.user_id.as_str().into(),
        Cell::opt_text(r.user_label.as_deref()),
        r.group_id
            .as_ref()
            .map_or("unattributed", GroupId::as_str)
            .into(),
        Cell::opt_text(r.group_name.as_deref()),
        r.project_id
            .as_ref()
            .map_or("unattributed", ProjectId::as_str)
            .into(),
        Cell::opt_text(r.project_name.as_deref()),
        r.provider.as_str().into(),
        r.model.as_str().into(),
        r.status.as_str().into(),
        Cell::opt_int(r.input_tokens),
        Cell::opt_int(r.output_tokens),
        Cell::Money(r.cost_microdollars),
        Cell::opt_int(r.latency_ms),
        r.tool_call_count.into(),
        r.decision_count.into(),
        r.deny_count.into(),
        Cell::opt_text(r.session_id.as_ref().map(SessionId::as_str)),
        Cell::opt_text(r.trace_id.as_ref().map(TraceId::as_str)),
        Cell::opt_text(r.error_message.as_deref()),
    ]
}

pub(crate) fn time_range(ctx: &ExportContext<'_>) -> AdminResult<TimeRange> {
    let w = ctx.window()?;
    Ok(TimeRange {
        from: w.from,
        to: w.to,
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    })
}

#[async_trait]
impl DataSet for Requests {
    fn id(&self) -> &'static str {
        "requests"
    }
    fn title(&self) -> &'static str {
        "Inference requests"
    }
    fn description(&self) -> &'static str {
        "One row per /v1/messages call: person, scope, model, status, tokens, cost and latency."
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
