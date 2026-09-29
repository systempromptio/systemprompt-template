//! Skill runs and each kit release's run record — the figures a kit
//! repository's CI reads to show what a release changed.
//!
//! Both are filtered by `marketplace=<id>` or `skill=<plugin:skill>` (one is
//! required) and carry no person, prompt or transcript: a row is a run's
//! timings, counts, cost and verdict, keyed by its conversation.

use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use super::analysis_skills::window;
use crate::error::{AdminError, AdminResult};
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::parse_skill_key;
use crate::handlers::ssr::analysis::skill_runs::{
    ReleaseImpact, agent_s, human_wait_s, release_impact, run_success, version_label,
};
use crate::repositories::analysis::skills::{SkillRunFilter, SkillRunRow, list_skill_runs};
use crate::types::UserContext;

pub(crate) struct SkillRuns;
pub(crate) struct KitReleaseImpact;

// Why: the request carries `format=`, `columns=` and the window beside these,
// so unknown keys are expected; `per_skill=false` folds a release's skills
// into one row.
#[derive(Debug, Default, Deserialize)]
struct RunScope {
    marketplace: Option<String>,
    skill: Option<String>,
    per_skill: Option<bool>,
}

impl RunScope {
    fn filter(&self) -> AdminResult<SkillRunFilter> {
        let marketplace = self
            .marketplace
            .as_deref()
            .map(str::trim)
            .filter(|m| !m.is_empty())
            .map(str::to_owned);
        let skill = match self
            .skill
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(raw) => Some(parse_skill_key(raw)?.skill),
            None => None,
        };
        if marketplace.is_none() && skill.is_none() {
            return Err(AdminError::BadRequest(
                "Name a marketplace= or a skill=plugin:skill".into(),
            ));
        }
        Ok(SkillRunFilter { marketplace, skill })
    }
}

// Why: `list_skill_runs` reads at most 5,000 runs. The file holds one fewer,
// so a 5,000th row is the proof there were more and the preview says capped;
// both datasets read the whole allowance whatever `ctx.limit` is, because the
// preview's one-row limit would otherwise count one run and fold one run
// into the release rows.
const RUN_READ: i64 = 5_000;
const RUN_CAP: i64 = RUN_READ - 1;

async fn load_runs(ctx: &ExportContext<'_>) -> AdminResult<(RunScope, Vec<SkillRunRow>)> {
    let scope: RunScope = ctx.query()?;
    let filter = scope.filter()?;
    let runs = list_skill_runs(ctx.pool, &window(ctx).await?, &filter, RUN_READ).await?;
    Ok((scope, runs))
}

const RUN_COLUMNS: &[Column] = &[
    Column::new("context_id", "Context", CellKind::Text).group("Identity"),
    Column::new("skill", "Entry skill", CellKind::Text).group("Identity"),
    Column::new("marketplace_id", "Marketplace", CellKind::Text).group("Identity"),
    Column::new("kit_version", "Kit version", CellKind::Text).group("Identity"),
    Column::new("kit_hash", "Kit hash", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("started_at", "Started", CellKind::Timestamp).group("Time"),
    Column::new("ended_at", "Ended", CellKind::Timestamp).group("Time"),
    Column::new("wall_s", "Wall (s)", CellKind::Decimal).group("Time"),
    Column::new("agent_s", "Agent (s)", CellKind::Decimal).group("Time"),
    Column::new("human_wait_s", "Human wait (s)", CellKind::Decimal).group("Time"),
    Column::new("prompts", "Prompts", CellKind::Integer).group("People"),
    Column::new("questions", "Questions asked", CellKind::Integer).group("People"),
    Column::new("turns", "Turns", CellKind::Integer).group("Volume"),
    Column::new("tool_calls", "Tool calls", CellKind::Integer).group("Volume"),
    Column::new("tool_failures", "Tool failures", CellKind::Integer).group("Volume"),
    Column::new("errors", "Failed requests", CellKind::Integer).group("Volume"),
    Column::new("rejected", "Rejected requests", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("contexts", "Contexts", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money).group("Cost"),
    Column::new("terminal_state", "Reached state", CellKind::Text).group("Outcome"),
    Column::new("transitions", "Logged transitions", CellKind::Integer)
        .optional()
        .group("Outcome"),
    Column::new("outcome", "Judge outcome", CellKind::Text).group("Outcome"),
    Column::new("completion", "Judge completion", CellKind::Integer).group("Outcome"),
    Column::new("success", "Succeeded", CellKind::Bool).group("Outcome"),
    Column::new("requirements_s", "Requirements (s)", CellKind::Decimal).group("Phases"),
    Column::new("design_s", "Design (s)", CellKind::Decimal).group("Phases"),
    Column::new("build_s", "Build (s)", CellKind::Decimal).group("Phases"),
    Column::new("verify_s", "Verify (s)", CellKind::Decimal).group("Phases"),
    Column::new("ship_s", "Ship (s)", CellKind::Decimal).group("Phases"),
];

fn run_row(r: &SkillRunRow, now: chrono::DateTime<Utc>) -> Vec<Cell> {
    vec![
        r.context_id.as_str().into(),
        r.skill.as_str().into(),
        Cell::opt_text(r.marketplace_id.as_ref().map(|m| m.as_str().to_owned())),
        version_label(r).into(),
        Cell::opt_text(r.kit_hash.as_deref()),
        r.started_at.into(),
        r.ended_at.into(),
        Cell::Decimal(r.wall_s),
        Cell::Decimal(agent_s(r)),
        Cell::Decimal(human_wait_s(r)),
        r.prompts.into(),
        r.questions.into(),
        r.turns.into(),
        r.tool_calls.into(),
        r.tool_failures.into(),
        r.errors.into(),
        r.rejected.into(),
        r.contexts.into(),
        Cell::Money(r.cost_microdollars),
        Cell::opt_text(r.terminal_state.as_deref()),
        r.transitions.into(),
        Cell::opt_text(r.outcome.as_deref()),
        Cell::opt_int(r.completion),
        run_success(r, now).map_or(Cell::Empty, Cell::Bool),
        Cell::opt_decimal(r.requirements_s),
        Cell::opt_decimal(r.design_s),
        Cell::opt_decimal(r.build_s),
        Cell::opt_decimal(r.verify_s),
        Cell::opt_decimal(r.ship_s),
    ]
}

#[async_trait]
impl DataSet for SkillRuns {
    fn id(&self) -> &'static str {
        "analysis-skill-runs"
    }
    fn title(&self) -> &'static str {
        "Skill runs"
    }
    fn description(&self) -> &'static str {
        "One row per session that ran a skill: time, human wait, prompts, cost and whether it finished."
    }
    fn columns(&self) -> &'static [Column] {
        RUN_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    fn cap(&self) -> i64 {
        RUN_CAP
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (_, runs) = load_runs(ctx).await?;
        let now = Utc::now();
        let total = i64::try_from(runs.len()).unwrap_or(i64::MAX);
        Ok(Table::capped_at(
            runs.iter()
                .take(usize::try_from(RUN_CAP).unwrap_or(usize::MAX))
                .map(|r| run_row(r, now))
                .collect(),
            total,
        ))
    }
}

const RELEASE_COLUMNS: &[Column] = &[
    Column::new("kit_version", "Kit version", CellKind::Text).group("Identity"),
    Column::new("skill", "Skill", CellKind::Text).group("Identity"),
    Column::new("first_run_at", "First run", CellKind::Timestamp).group("Identity"),
    Column::new("runs", "Runs", CellKind::Integer).group("Volume"),
    Column::new("succeeded", "Succeeded", CellKind::Integer).group("Outcome"),
    Column::new("failed", "Failed", CellKind::Integer).group("Outcome"),
    Column::new("unknown", "Unknown", CellKind::Integer).group("Outcome"),
    Column::new("success_rate", "Success rate", CellKind::Decimal).group("Outcome"),
    Column::new("abandoned", "Judged abandoned", CellKind::Integer)
        .optional()
        .group("Outcome"),
    Column::new("median_wall_s", "Median wall (s)", CellKind::Decimal).group("Time"),
    Column::new("p90_wall_s", "p90 wall (s)", CellKind::Decimal).group("Time"),
    Column::new("median_agent_s", "Median agent (s)", CellKind::Decimal).group("Time"),
    Column::new("median_wait_s", "Median human wait (s)", CellKind::Decimal).group("Time"),
    Column::new("median_prompts", "Median prompts", CellKind::Decimal).group("People"),
    Column::new("median_questions", "Median questions", CellKind::Decimal).group("People"),
    Column::new("tool_failure_rate", "Tool failure rate", CellKind::Decimal).group("Volume"),
    Column::new("cost_per_run_usd", "Cost per run (USD)", CellKind::Money).group("Cost"),
];

fn release_row(r: &ReleaseImpact) -> Vec<Cell> {
    vec![
        r.kit_version.as_str().into(),
        Cell::opt_text(r.skill.as_deref()),
        r.first_run_at.into(),
        r.runs.into(),
        r.succeeded.into(),
        r.failed.into(),
        r.unknown.into(),
        Cell::opt_decimal(r.success_rate),
        r.abandoned.into(),
        Cell::Decimal(r.median_wall_s),
        Cell::Decimal(r.p90_wall_s),
        Cell::Decimal(r.median_agent_s),
        Cell::Decimal(r.median_wait_s),
        Cell::Decimal(r.median_prompts),
        Cell::Decimal(r.median_questions),
        Cell::opt_decimal(r.tool_failure_rate),
        Cell::Money(r.cost_per_run_microdollars),
    ]
}

#[async_trait]
impl DataSet for KitReleaseImpact {
    fn id(&self) -> &'static str {
        "analysis-kit-release-impact"
    }
    fn title(&self) -> &'static str {
        "Kit release impact"
    }
    fn description(&self) -> &'static str {
        "One row per kit version and skill: runs, success rate, median time, human wait and cost per run."
    }
    fn columns(&self) -> &'static [Column] {
        RELEASE_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (scope, runs) = load_runs(ctx).await?;
        let rows = release_impact(&runs, scope.per_skill.unwrap_or(true), Utc::now());
        Ok(Table::complete(rows.iter().map(release_row).collect()))
    }
}
