//! `/admin/history` (my conversations) and `/admin/conversations` (everyone's).

use async_trait::async_trait;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_history::{ExportRequest, HistoryView, export_rows};
use crate::repositories::analytics::conversations::HistoryItem;
use crate::types::UserContext;
use systemprompt::identifiers::{ContextId, SessionId};

pub(crate) struct History;
pub(crate) struct OrgHistory;

const DESCRIPTION: &str =
    "One row per conversation — who, model, turns, tokens, cost — by last activity.";

const COLUMNS: &[Column] = &[
    Column::new("started_at", "Started", CellKind::Timestamp),
    Column::new("last_at", "Last activity", CellKind::Timestamp),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("context_id", "Context", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("user_label", "Name", CellKind::Text),
    Column::new("title", "Title", CellKind::Text),
    Column::new("preview", "Preview", CellKind::Text).optional(),
    Column::new("model", "Model", CellKind::Text),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("side_calls", "Side calls", CellKind::Integer),
];

fn row(r: &HistoryItem) -> Vec<Cell> {
    vec![
        Cell::opt_time(r.started_at),
        r.last_at.into(),
        Cell::opt_text(r.session_id.as_ref().map(SessionId::as_str)),
        Cell::opt_text(r.context_id.as_ref().map(ContextId::as_str)),
        r.user_id.as_str().into(),
        r.user_label.as_str().into(),
        Cell::opt_text(r.title.as_deref()),
        Cell::opt_text(r.preview.as_deref()),
        Cell::opt_text(r.model.as_deref()),
        r.turns.into(),
        r.total_input_tokens.into(),
        r.total_output_tokens.into(),
        Cell::Money(r.cost_microdollars),
        r.side_call_count.into(),
    ]
}

async fn load(ctx: &ExportContext<'_>, view: HistoryView) -> AdminResult<Table> {
    let (rows, total) = export_rows(
        ctx.pool,
        ctx.user,
        ExportRequest {
            query: ctx.query()?,
            view,
            limit: ctx.limit,
        },
    )
    .await?;
    Ok(Table {
        rows: rows.iter().map(row).collect(),
        total,
    })
}

#[async_trait]
impl DataSet for History {
    fn id(&self) -> &'static str {
        "history"
    }
    fn title(&self) -> &'static str {
        "My conversations"
    }
    fn description(&self) -> &'static str {
        DESCRIPTION
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::None
    }
    // Why: identity-scoped like the page — every signed-in user may export
    // their own history; the handler narrows the rows to their scope.
    fn allows(&self, _user: &UserContext) -> bool {
        true
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        load(ctx, HistoryView::Own).await
    }
}

#[async_trait]
impl DataSet for OrgHistory {
    fn id(&self) -> &'static str {
        "conversations"
    }
    fn title(&self) -> &'static str {
        "Conversations"
    }
    fn description(&self) -> &'static str {
        DESCRIPTION
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::None
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        load(ctx, HistoryView::Org).await
    }
}
