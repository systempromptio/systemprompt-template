//! `/admin/analysis/skills/{key}` — the conversations that invoked one
//! skill over the window.

use async_trait::async_trait;
use serde::Deserialize;

use super::analysis_skills::window;
use crate::error::{AdminError, AdminResult};
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::parse_skill_key;
use crate::repositories::analysis::skills::{SkillConversationRow, list_skill_conversations_paged};
use crate::types::UserContext;

pub(crate) struct SkillConversations;

// Why: the export dialog is opened from the page, so the request's query
// carries `format=`, `columns=` and the window beside the drill-down;
// `skill` is the `plugin:skill` key the page carries in its path.
#[derive(Debug, Default, Deserialize)]
struct Drilldown {
    skill: Option<String>,
}

const CONVERSATION_COLUMNS: &[Column] = &[
    Column::new("context_id", "Context", CellKind::Text).group("Identity"),
    Column::new("client_session_id", "Harness session", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("title", "Title", CellKind::Text).group("Identity"),
    Column::new("user_id", "User", CellKind::Text).group("Identity"),
    Column::new("display_name", "Name", CellKind::Text).group("Identity"),
    Column::new("group_name", "Group", CellKind::Text).group("Identity"),
    Column::new("project_name", "Project", CellKind::Text).group("Identity"),
    Column::new("client", "Client", CellKind::Text).group("Identity"),
    Column::new("model", "Model", CellKind::Text).group("Identity"),
    Column::new("invocations", "Invocations", CellKind::Integer).group("Volume"),
    Column::new("turns", "Turns", CellKind::Integer).group("Volume"),
    Column::new("tool_calls", "Tool calls", CellKind::Integer).group("Tools & artifacts"),
    Column::new("errors", "Failed requests", CellKind::Integer).group("Volume"),
    Column::new("denied", "Denied", CellKind::Integer).group("Governance"),
    Column::new("tokens", "Tokens", CellKind::Integer).group("Tokens"),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money).group("Cost"),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("duration_seconds", "Duration (s)", CellKind::Integer).group("Volume"),
    Column::new("skills", "Skills", CellKind::Text)
        .optional()
        .group("Tools & artifacts"),
    Column::new("category", "Intent", CellKind::Text).group("Judge"),
    Column::new("outcome", "Outcome", CellKind::Text).group("Judge"),
    Column::new("completion", "Completion", CellKind::Integer).group("Judge"),
    Column::new("first_at", "Started", CellKind::Timestamp).group("Identity"),
    Column::new("last_at", "Last activity", CellKind::Timestamp).group("Identity"),
];

fn conversation_row(r: &SkillConversationRow) -> Vec<Cell> {
    vec![
        r.context_id.as_str().into(),
        Cell::opt_text(r.client_session_id.as_deref()),
        Cell::opt_text(r.judge_title.as_deref().or(Some(r.title.as_str()))),
        r.user_id.as_str().into(),
        Cell::opt_text(r.display_name.as_deref()),
        Cell::opt_text(r.group_name.as_deref()),
        Cell::opt_text(r.project_name.as_deref()),
        r.client_kind.as_str().into(),
        Cell::opt_text(r.model.as_deref()),
        r.invocations.into(),
        r.turn_count.into(),
        r.tool_calls.into(),
        r.error_count.into(),
        r.gov_deny.into(),
        r.total_tokens.into(),
        Cell::Money(r.cost_microdollars),
        Cell::opt_int(r.p95_latency_ms),
        r.duration_seconds.into(),
        Cell::list(&r.skills),
        Cell::opt_text(r.category.as_deref()),
        Cell::opt_text(r.outcome.as_deref()),
        Cell::opt_int(r.completion),
        Cell::opt_time(Some(r.first_at)),
        Cell::opt_time(Some(r.last_at)),
    ]
}

#[async_trait]
impl DataSet for SkillConversations {
    fn id(&self) -> &'static str {
        "analysis-skill-conversations"
    }
    fn title(&self) -> &'static str {
        "Skill conversations"
    }
    fn description(&self) -> &'static str {
        "One row per conversation that invoked the skill."
    }
    fn columns(&self) -> &'static [Column] {
        CONVERSATION_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let drill: Drilldown = ctx.query()?;
        let skill = drill
            .skill
            .as_deref()
            .ok_or_else(|| AdminError::BadRequest("A skill is required".into()))?;
        let skill = parse_skill_key(skill)?;
        let (rows, total) = list_skill_conversations_paged(
            ctx.pool,
            &window(ctx).await?,
            &skill.skill,
            ctx.limit,
            0,
        )
        .await?;
        Ok(Table {
            rows: rows.iter().map(conversation_row).collect(),
            total,
        })
    }
}
