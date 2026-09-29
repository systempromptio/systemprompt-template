//! `/admin/tools` and `/admin/artifacts` — every tool call in the window as
//! the ledger shows it, the artifact subset, and the breakdown buckets over
//! the same filtered set. Both lenses share one row shape; `ids=` narrows
//! any of them to a selection made on the page.

use async_trait::async_trait;

use super::requests::time_range;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_tools::{Lens, export_rows};
use crate::repositories::analysis::tools::{ToolActivityRow, ToolBucketRow};
use systemprompt::identifiers::UserId;

pub(crate) struct Tools;
pub(crate) struct ToolsBreakdown;
pub(crate) struct Artifacts;
pub(crate) struct ArtifactsBreakdown;

const COLUMNS: &[Column] = &[
    Column::new("occurred_at", "Time", CellKind::Timestamp),
    Column::new("row_key", "Call", CellKind::Text),
    Column::new("tool", "Tool", CellKind::Text),
    Column::new("server", "Server", CellKind::Text),
    Column::new("origin", "Origin", CellKind::Text),
    Column::new("input", "Input", CellKind::Text),
    Column::new("state", "State", CellKind::Text),
    Column::new("status", "Execution status", CellKind::Text).optional(),
    Column::new("duration_ms", "Duration (ms)", CellKind::Integer),
    Column::new("decision", "Decision", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("user_label", "Name", CellKind::Text),
    Column::new("client", "Client", CellKind::Text).optional(),
    Column::new("source", "Source", CellKind::Text).optional(),
    Column::new("context_id", "Conversation", CellKind::Text),
    Column::new("session_id", "Harness session", CellKind::Text).optional(),
    Column::new("request_id", "Request", CellKind::Text).optional(),
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("artifact_kind", "Artifact kind", CellKind::Text),
    Column::new("artifact_id", "Artifact", CellKind::Text).optional(),
    Column::new("artifact_type", "Artifact type", CellKind::Text).optional(),
    Column::new("artifact_title", "Title", CellKind::Text),
    Column::new("bytes", "Payload bytes", CellKind::Integer).optional(),
    Column::new("redactions", "Secret redactions", CellKind::Integer).optional(),
    Column::new("error", "Error", CellKind::Bool),
    Column::new("error_message", "Error message", CellKind::Text).optional(),
];

fn row(r: &ToolActivityRow) -> Vec<Cell> {
    vec![
        Cell::opt_time(r.occurred_at),
        r.row_key.as_str().into(),
        Cell::opt_text(r.tool_name.as_deref()),
        Cell::opt_text(r.server_name.as_deref()),
        if r.is_builtin { "builtin" } else { "mcp" }.into(),
        Cell::opt_text(r.input_summary.as_deref()),
        if r.failed {
            "failed".into()
        } else {
            r.state.as_str().into()
        },
        Cell::opt_text(r.execution_status.as_deref()),
        Cell::opt_int(r.execution_time_ms),
        Cell::opt_text(r.decision.as_deref()),
        Cell::opt_text(r.user_id.as_ref().map(UserId::as_str)),
        Cell::opt_text(r.display_name.as_deref()),
        Cell::opt_text(r.client_kind.as_deref()),
        Cell::opt_text(r.source.as_deref()),
        Cell::opt_text(
            r.context_key
                .as_deref()
                .or(r.execution_context_key.as_deref()),
        ),
        Cell::opt_text(r.execution_trace_id.as_deref().or(r.session_key.as_deref())),
        Cell::opt_text(r.request_id.as_deref()),
        Cell::opt_text(r.skill.as_deref()),
        Cell::opt_text(r.artifact_kind.as_deref()),
        Cell::opt_text(r.artifact_key.as_deref()),
        Cell::opt_text(r.artifact_type.as_deref()),
        Cell::opt_text(r.artifact_title.as_deref()),
        Cell::opt_int(r.payload_bytes),
        Cell::opt_int(r.secret_redactions),
        r.failed.into(),
        Cell::opt_text(r.error_message.as_deref()),
    ]
}

const BUCKET_COLUMNS: &[Column] = &[
    Column::new("bucket", "Bucket", CellKind::Text),
    Column::new("value", "Value", CellKind::Text).optional(),
    Column::new("calls", "Calls", CellKind::Integer),
    Column::new("executed", "Executed", CellKind::Integer),
    Column::new("failed", "Failed", CellKind::Integer),
    Column::new("denied", "Denied", CellKind::Integer),
    Column::new("users", "Users", CellKind::Integer),
    Column::new("artifacts", "Artifacts", CellKind::Integer),
    Column::new("p95_duration_ms", "p95 duration (ms)", CellKind::Decimal).optional(),
    Column::new("bytes", "Payload bytes", CellKind::Integer).optional(),
];

fn bucket_row(b: &ToolBucketRow) -> Vec<Cell> {
    vec![
        b.label.as_str().into(),
        Cell::opt_text(b.value.as_deref()),
        b.calls.into(),
        b.executed.into(),
        b.failed.into(),
        b.denied.into(),
        b.users.into(),
        b.artifacts.into(),
        Cell::opt_decimal(b.p95_duration_ms),
        b.bytes.into(),
    ]
}

async fn rows(ctx: &ExportContext<'_>, lens: Lens) -> AdminResult<Table> {
    let data = export_rows(
        ctx.pool,
        ctx.user,
        lens,
        ctx.query()?,
        (time_range(ctx)?, ctx.limit),
    )
    .await?;
    let total = match lens {
        Lens::Tools => data.totals.calls,
        Lens::Artifacts => data.totals.artifacts,
    };
    Ok(Table {
        rows: data.rows.iter().map(row).collect(),
        total,
    })
}

async fn buckets(ctx: &ExportContext<'_>, lens: Lens) -> AdminResult<Table> {
    let data = export_rows(
        ctx.pool,
        ctx.user,
        lens,
        ctx.query()?,
        (time_range(ctx)?, 1),
    )
    .await?;
    Ok(Table::complete(
        data.breakdown.iter().map(bucket_row).collect(),
    ))
}

#[async_trait]
impl DataSet for Tools {
    fn id(&self) -> &'static str {
        "tools"
    }
    fn title(&self) -> &'static str {
        "Tool calls"
    }
    fn description(&self) -> &'static str {
        "One row per tool call in the window: tool, server, outcome, decision, person and duration."
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        rows(ctx, Lens::Tools).await
    }
}

#[async_trait]
impl DataSet for ToolsBreakdown {
    fn id(&self) -> &'static str {
        "tools-breakdown"
    }
    fn title(&self) -> &'static str {
        "Tool call breakdown"
    }
    fn description(&self) -> &'static str {
        "One row per bucket of the chosen dimension: calls, failures, denials, people and artifacts."
    }
    fn columns(&self) -> &'static [Column] {
        BUCKET_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        buckets(ctx, Lens::Tools).await
    }
}

#[async_trait]
impl DataSet for Artifacts {
    fn id(&self) -> &'static str {
        "artifacts"
    }
    fn title(&self) -> &'static str {
        "Artifacts"
    }
    fn description(&self) -> &'static str {
        "One row per tool call that produced an artifact: the call, its artifact and size."
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        rows(ctx, Lens::Artifacts).await
    }
}

#[async_trait]
impl DataSet for ArtifactsBreakdown {
    fn id(&self) -> &'static str {
        "artifacts-breakdown"
    }
    fn title(&self) -> &'static str {
        "Artifact breakdown"
    }
    fn description(&self) -> &'static str {
        "One row per bucket of the chosen dimension over the artifact-producing calls."
    }
    fn columns(&self) -> &'static [Column] {
        BUCKET_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        buckets(ctx, Lens::Artifacts).await
    }
}
