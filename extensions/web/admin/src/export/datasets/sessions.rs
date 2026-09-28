//! `/admin/sessions` — one row per conversation in the window.

use async_trait::async_trait;

use super::requests::time_range;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_sessions_list::export::export_rows;
use crate::repositories::analytics::conversation_rows::ConversationRow;
use systemprompt::identifiers::{SessionId, UserId};

pub(crate) struct Sessions;

pub(crate) const COLUMNS: &[Column] = &[
    Column::new("first_at", "Started", CellKind::Timestamp),
    Column::new("last_at", "Last activity", CellKind::Timestamp),
    Column::new("context_id", "Context", CellKind::Text),
    Column::new("session_id", "Session", CellKind::Text).optional(),
    Column::new("client_session_id", "Client session", CellKind::Text).optional(),
    Column::new("title", "Title", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("display_name", "Name", CellKind::Text),
    Column::new("group_name", "Group", CellKind::Text),
    Column::new("project_name", "Project", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("status", "Status", CellKind::Text),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("side_calls", "Side calls", CellKind::Integer),
    Column::new("tool_calls", "Tool calls", CellKind::Integer),
    Column::new("errors", "Errors", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new(
        "side_call_cost_usd",
        "Side-call cost (USD)",
        CellKind::Money,
    )
    .optional(),
];

pub(crate) fn conversation_row(r: &ConversationRow) -> Vec<Cell> {
    vec![
        Cell::opt_time(r.first_at),
        Cell::opt_time(r.last_at),
        r.context_id.as_str().into(),
        Cell::opt_text(r.session_id.as_ref().map(SessionId::as_str)),
        Cell::opt_text(r.client_session_id.as_deref()),
        r.title.as_str().into(),
        Cell::opt_text(r.user_id.as_ref().map(UserId::as_str)),
        Cell::opt_text(r.display_name.as_deref()),
        Cell::opt_text(r.group_name.as_deref()),
        Cell::opt_text(r.project_name.as_deref()),
        Cell::opt_text(r.model.as_deref()),
        Cell::opt_text(r.status.as_deref()),
        r.turn_count.into(),
        r.side_call_count.into(),
        r.tool_call_count.into(),
        r.error_count.into(),
        r.total_input_tokens.into(),
        r.total_output_tokens.into(),
        Cell::Money(r.total_cost_microdollars),
        Cell::Money(r.side_call_cost_microdollars),
    ]
}

#[async_trait]
impl DataSet for Sessions {
    fn id(&self) -> &'static str {
        "sessions"
    }
    fn title(&self) -> &'static str {
        "Sessions"
    }
    fn description(&self) -> &'static str {
        "One row per conversation in the window: person, model, turns, tokens and cost."
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
            rows: rows.iter().map(conversation_row).collect(),
            total,
        })
    }
}
