//! The projects listing as a file: one row per project with the same
//! membership, spend, model and agent mix, tool health and output the page
//! shows, over the window the dialog picks.

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories::projects::crud::list_projects;
use crate::repositories::projects::usage::{LISTING_CAP, ProjectRollup, list_project_rollups};

pub(crate) struct Projects;

#[derive(Debug, Default, Deserialize)]
struct ProjectsQuery {
    q: Option<String>,
}

fn matches_search(needle: &str, id: &str, name: &str) -> bool {
    needle.is_empty() || name.to_lowercase().contains(needle) || id.to_lowercase().contains(needle)
}

const COLUMNS: &[Column] = &[
    Column::new("id", "Project id", CellKind::Text).group("Identity"),
    Column::new("name", "Name", CellKind::Text).group("Identity"),
    Column::new("description", "Description", CellKind::Text).group("Identity"),
    Column::new("members", "Members", CellKind::Integer).group("People"),
    Column::new(
        "attributed_members",
        "Attributed members",
        CellKind::Integer,
    )
    .group("People")
    .optional(),
    Column::new("active_members", "Active members", CellKind::Integer).group("People"),
    Column::new("groups", "Groups", CellKind::Integer).group("People"),
    Column::new("requests", "Requests", CellKind::Integer).group("Usage"),
    Column::new("tokens", "Tokens", CellKind::Integer).group("Usage"),
    Column::new("cost", "Cost", CellKind::Money).group("Usage"),
    Column::new("models", "Models", CellKind::Integer).group("Mix"),
    Column::new("top_model", "Top model", CellKind::Text).group("Mix"),
    Column::new("agents", "Agents", CellKind::Integer).group("Mix"),
    Column::new("top_agent", "Top agent", CellKind::Text).group("Mix"),
    Column::new("tool_calls", "Tool calls", CellKind::Integer).group("Tools"),
    Column::new("tool_success", "Tool successes", CellKind::Integer).group("Tools"),
    Column::new("tool_success_pct", "Tool success %", CellKind::Integer).group("Tools"),
    Column::new("skills", "Skills", CellKind::Integer).group("Output"),
    Column::new("artifacts", "Artifacts", CellKind::Integer).group("Output"),
];

fn row(p: &ProjectRollup) -> Vec<Cell> {
    let success_pct = if p.tool_calls > 0 {
        p.tool_success * 100 / p.tool_calls
    } else {
        0
    };
    vec![
        p.id.as_str().into(),
        p.name.as_str().into(),
        Cell::opt_text(p.description.as_deref()),
        p.member_count.into(),
        p.attributed_members.into(),
        p.active_members.into(),
        p.group_count.into(),
        p.requests.into(),
        p.tokens.into(),
        Cell::Money(p.cost_microdollars),
        p.models_used.into(),
        Cell::opt_text(p.top_model.as_deref()),
        p.clients_used.into(),
        Cell::opt_text(p.top_client.as_deref()),
        p.tool_calls.into(),
        p.tool_success.into(),
        success_pct.into(),
        p.skills_used.into(),
        p.artifacts.into(),
    ]
}

#[async_trait]
impl DataSet for Projects {
    fn id(&self) -> &'static str {
        "projects"
    }
    fn title(&self) -> &'static str {
        "Projects"
    }
    fn description(&self) -> &'static str {
        "One row per project: members, groups, requests, tokens, cost, model and agent mix, tool health, skills and artifacts."
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Days
    }
    fn cap(&self) -> i64 {
        LISTING_CAP
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let query: ProjectsQuery = ctx.query()?;
        let window = ctx.window()?;
        // Why: the rollup counts back from now in whole days, so the dialog's
        // day preset is the number of days it spans.
        let days = i32::try_from((window.to - window.from).num_days().max(1)).unwrap_or(30);
        let needle = query.q.unwrap_or_default().trim().to_lowercase();
        // Why: the rollup stops at `LISTING_CAP` projects by name; the plain
        // project list is what says how many the search matched, so a cut
        // file reads as capped rather than complete.
        let (rows, all) = tokio::try_join!(
            list_project_rollups(ctx.pool, days, LISTING_CAP),
            list_projects(ctx.pool),
        )?;
        let total = all
            .iter()
            .filter(|p| matches_search(&needle, p.id.as_str(), &p.name))
            .count();
        Ok(Table::capped_at(
            rows.iter()
                .filter(|p| matches_search(&needle, p.id.as_str(), &p.name))
                .map(row)
                .collect(),
            i64::try_from(total).unwrap_or(i64::MAX),
        ))
    }
}
