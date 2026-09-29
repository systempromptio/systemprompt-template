//! `/admin/analysis/conversations/{context_id}` — one conversation as a
//! ledger: every request, every tool call and every artifact in time order,
//! one row each, so a single conversation can leave the console as one file.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::{ArtifactId, ContextId};

use crate::error::{AdminError, AdminResult};
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories::analysis::conversations::detail::find_conversation_facts;
use crate::repositories::analysis::conversations::planes::{
    ConversationToolCallRow, ConversationTurnRow, list_conversation_tool_calls,
    list_conversation_turns,
};
use crate::repositories::scope::visibility::may_view;
use crate::types::UserContext;

pub(crate) struct ConversationTurns;

const COLUMNS: &[Column] = &[
    Column::new("at", "Time", CellKind::Timestamp).group("Identity"),
    Column::new("kind", "Row", CellKind::Text).group("Identity"),
    Column::new("context_id", "Context", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("request_id", "Request", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("label", "Model / tool", CellKind::Text).group("Identity"),
    Column::new("detail", "Detail", CellKind::Text).group("Identity"),
    Column::new("status", "Status", CellKind::Text).group("Volume"),
    Column::new("input_tokens", "Input tokens", CellKind::Integer).group("Tokens"),
    Column::new("output_tokens", "Output tokens", CellKind::Integer).group("Tokens"),
    Column::new("cache_tokens", "Cache tokens", CellKind::Integer)
        .optional()
        .group("Tokens"),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money).group("Cost"),
    Column::new("duration_ms", "Duration (ms)", CellKind::Integer).group("Volume"),
    Column::new("artifact_kind", "Artifact kind", CellKind::Text).group("Tools & artifacts"),
    Column::new("artifact", "Artifact", CellKind::Text)
        .optional()
        .group("Tools & artifacts"),
    Column::new("error", "Error", CellKind::Text)
        .optional()
        .group("Governance"),
];

#[derive(Debug, Default, serde::Deserialize)]
struct ConversationQuery {
    #[serde(rename = "context_id")]
    context: Option<String>,
}

fn turn_row(context: &str, r: &ConversationTurnRow) -> (DateTime<Utc>, Vec<Cell>) {
    let cells = vec![
        r.created_at.into(),
        r.effective_kind.as_str().into(),
        context.into(),
        r.request_id.as_str().into(),
        Cell::opt_text(r.model.as_deref()),
        Cell::list(&r.tool_names),
        r.status.as_str().into(),
        r.input_tokens.into(),
        r.output_tokens.into(),
        (r.cache_read_tokens + r.cache_creation_tokens).into(),
        Cell::Money(r.cost_microdollars),
        Cell::opt_int(r.latency_ms),
        Cell::Empty,
        Cell::Empty,
        Cell::opt_text(r.error_message.as_deref()),
    ];
    (r.created_at, cells)
}

fn tool_row(context: &str, r: &ConversationToolCallRow) -> (DateTime<Utc>, Vec<Cell>) {
    let at = r.occurred_at.unwrap_or_default();
    let kind = if r.artifact_kind.is_some() {
        "artifact"
    } else {
        "tool"
    };
    let cells = vec![
        Cell::opt_time(r.occurred_at),
        kind.into(),
        context.into(),
        Cell::opt_text(r.request_id.as_deref()),
        Cell::opt_text(r.tool_name.as_deref()),
        Cell::opt_text(r.input_summary.as_deref()),
        r.execution_status
            .as_deref()
            .unwrap_or(r.state.as_str())
            .into(),
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::opt_int(r.execution_time_ms),
        Cell::opt_text(r.artifact_kind.as_deref()),
        Cell::opt_text(
            r.artifact_title
                .as_deref()
                .or_else(|| r.artifact_id.as_ref().map(ArtifactId::as_str)),
        ),
        Cell::opt_text(r.error_message.as_deref()),
    ];
    (at, cells)
}

#[async_trait]
impl DataSet for ConversationTurns {
    fn id(&self) -> &'static str {
        "analysis-conversation-turns"
    }
    fn title(&self) -> &'static str {
        "This conversation"
    }
    fn description(&self) -> &'static str {
        "Every request, tool call and artifact of one conversation, in time order."
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::None
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    // Why: the same 404-not-403 rule as the page — a context outside the
    // caller's view is indistinguishable from a missing one.
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let query: ConversationQuery = ctx.query()?;
        let raw = query
            .context
            .ok_or_else(|| AdminError::BadRequest("A context_id is required.".to_owned()))?;
        let context_id = ContextId::try_new(raw.trim())
            .map_err(|_bad| AdminError::BadRequest("That is not a conversation id.".to_owned()))?;
        let not_found = || AdminError::NotFound("No conversation matches that id.".to_owned());
        let facts = find_conversation_facts(ctx.pool, &context_id)
            .await?
            .ok_or_else(not_found)?;
        if !may_view(ctx.pool, ctx.user, Some(&facts.user_id)).await? {
            return Err(not_found());
        }
        let session = facts.client_session_id.as_deref();
        let turns = list_conversation_turns(ctx.pool, &context_id).await?;
        let tools = list_conversation_tool_calls(ctx.pool, &context_id, session).await?;
        let key = context_id.as_str();
        let mut rows: Vec<(DateTime<Utc>, Vec<Cell>)> = turns
            .iter()
            .map(|t| turn_row(key, t))
            .chain(tools.iter().map(|t| tool_row(key, t)))
            .collect();
        rows.sort_by_key(|(at, _)| *at);
        Ok(Table::complete(rows.into_iter().map(|(_, c)| c).collect()))
    }
}
