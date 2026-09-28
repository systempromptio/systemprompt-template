//! `/admin/contexts` — activity by person: every conversation, or one row
//! per person with their totals.

use async_trait::async_trait;

use super::sessions::conversation_row;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_skills_contexts::export::export_rows;
use crate::repositories::analytics::conversation_rows::{
    ConversationPageMode, UserConversationSummary,
};

pub(crate) struct Contexts;
pub(crate) struct People;

const PEOPLE_COLUMNS: &[Column] = &[
    Column::new("user_id", "User", CellKind::Text),
    Column::new("display_name", "Name", CellKind::Text),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("side_calls", "Side calls", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("last_at", "Last activity", CellKind::Timestamp),
    Column::new("models", "Models", CellKind::Text),
    Column::new("latest_title", "Latest conversation", CellKind::Text).optional(),
];

fn person_row(r: &UserConversationSummary) -> Vec<Cell> {
    vec![
        r.user_id.as_str().into(),
        Cell::opt_text(r.display_name.as_deref()),
        r.conversation_count.into(),
        r.turn_count.into(),
        r.side_call_count.into(),
        r.total_tokens.into(),
        Cell::Money(r.total_cost_microdollars),
        Cell::opt_time(r.last_at),
        Cell::list(&r.models),
        Cell::opt_text(r.latest.as_ref().map(|c| c.title.as_str())),
    ]
}

#[async_trait]
impl DataSet for Contexts {
    fn id(&self) -> &'static str {
        "contexts"
    }
    fn title(&self) -> &'static str {
        "Conversations by person"
    }
    fn description(&self) -> &'static str {
        "One row per conversation in the window: person, model, turns, tokens and cost."
    }
    fn columns(&self) -> &'static [Column] {
        super::sessions::COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let result = export_rows(ctx, ConversationPageMode::All).await?;
        Ok(Table {
            rows: result.conversations.iter().map(conversation_row).collect(),
            total: result.totals.conversations,
        })
    }
}

#[async_trait]
impl DataSet for People {
    fn id(&self) -> &'static str {
        "people"
    }
    fn title(&self) -> &'static str {
        "People"
    }
    fn description(&self) -> &'static str {
        "One row per person active in the window: conversations, turns, tokens, cost and latest conversation."
    }
    fn columns(&self) -> &'static [Column] {
        PEOPLE_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let result = export_rows(ctx, ConversationPageMode::Users).await?;
        Ok(Table {
            rows: result.user_summaries.iter().map(person_row).collect(),
            total: result.totals.users,
        })
    }
}
