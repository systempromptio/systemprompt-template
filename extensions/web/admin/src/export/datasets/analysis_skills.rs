//! `/admin/analysis/skills` — every skill's record over the window. The
//! conversations that invoked one skill are `analysis_skill_conversations`.

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories;
use crate::repositories::analysis::skills::{
    SkillFactRow, SkillListFilter, SkillWindow, list_skill_facts,
};
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use systemprompt::identifiers::{MarketplaceId, PluginId};

pub(crate) struct Skills;

// Why: the ticked rows' keys, comma-joined by the export dialog, and the
// page's filters — the marketplace, client and search the rows were
// narrowed by. `skill` is the page's alias for `search`.
#[derive(Debug, Default, Deserialize)]
struct SkillIdsQuery {
    ids: Option<String>,
    marketplace: Option<String>,
    client: Option<String>,
    search: Option<String>,
    skill: Option<String>,
}

// Why: the page's group and project scope, read by every skills dataset so a
// file covers the people the page counted.
#[derive(Debug, Default, Deserialize)]
struct ScopeQuery {
    group: Option<String>,
    project: Option<String>,
}

// Why: `list_skill_facts` returns at most 1,000 skills.
const SKILL_CAP: i64 = 1_000;

impl SkillIdsQuery {
    fn skills(&self) -> Option<Vec<String>> {
        let keys: Vec<String> = self
            .ids
            .as_deref()?
            .split(',')
            .filter_map(|k| trimmed(Some(k)))
            .collect();
        (!keys.is_empty()).then_some(keys)
    }
}

fn trimmed(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

pub(super) async fn window(ctx: &ExportContext<'_>) -> AdminResult<SkillWindow> {
    let w = ctx.window()?;
    let q: ScopeQuery = ctx.query()?;
    let request = ScopeRequest::from_query(ctx.user, q.group.as_deref(), q.project.as_deref());
    let scope = repositories::scope::membership::get_subject_scope(ctx.pool, &request).await?;
    Ok(SkillWindow {
        start: w.from,
        end: w.to,
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
    })
}

const SKILL_COLUMNS: &[Column] = &[
    Column::new("skill", "Skill", CellKind::Text).group("Identity"),
    Column::new("plugin_id", "Plugin", CellKind::Text).group("Identity"),
    Column::new("marketplace_id", "Marketplace", CellKind::Text).group("Identity"),
    Column::new("invocations", "Invocations", CellKind::Integer).group("Volume"),
    Column::new("slash", "Slash", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("tool", "Tool", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("attributed", "Attributed", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("users", "People", CellKind::Integer).group("Volume"),
    Column::new("sessions", "Sessions", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("installs", "Installs", CellKind::Integer).group("Volume"),
    Column::new("conversations", "Conversations", CellKind::Integer).group("Volume"),
    Column::new("requests", "Requests", CellKind::Integer).group("Volume"),
    Column::new("turns", "Turns", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("input_tokens", "Input tokens", CellKind::Integer).group("Tokens"),
    Column::new("output_tokens", "Output tokens", CellKind::Integer).group("Tokens"),
    Column::new("cache_tokens", "Cache tokens", CellKind::Integer)
        .optional()
        .group("Tokens"),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money).group("Cost"),
    Column::new("errors", "Failed requests", CellKind::Integer).group("Volume"),
    Column::new("denied", "Denied", CellKind::Integer).group("Governance"),
    Column::new("tool_calls", "Tool calls", CellKind::Integer).group("Tools & artifacts"),
    Column::new("tool_calls_failed", "Tools failed", CellKind::Integer)
        .optional()
        .group("Tools & artifacts"),
    Column::new("artifacts", "Artifacts", CellKind::Integer)
        .optional()
        .group("Tools & artifacts"),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Decimal)
        .optional()
        .group("Volume"),
    Column::new("models", "Models", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("clients", "Clients", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("judged", "Judged", CellKind::Integer).group("Judge"),
    Column::new("completion_avg", "Mean completion", CellKind::Decimal).group("Judge"),
    Column::new("achieved", "Achieved", CellKind::Integer)
        .optional()
        .group("Judge"),
    Column::new("first_used", "First used", CellKind::Timestamp).group("Identity"),
    Column::new("last_used", "Last used", CellKind::Timestamp).group("Identity"),
];

fn skill_row(r: &SkillFactRow) -> Vec<Cell> {
    vec![
        r.skill.as_str().into(),
        Cell::opt_text(r.plugin_id.as_ref().map(PluginId::as_str)),
        Cell::opt_text(r.marketplace_id.as_ref().map(MarketplaceId::as_str)),
        r.invocations.into(),
        r.slash.into(),
        r.tool.into(),
        r.attributed.into(),
        r.users.into(),
        r.sessions.into(),
        r.installs.into(),
        r.conversations.into(),
        r.requests.into(),
        r.turns.into(),
        r.input_tokens.into(),
        r.output_tokens.into(),
        r.cache_tokens.into(),
        Cell::Money(r.cost_microdollars),
        r.errors.into(),
        r.denied.into(),
        r.tool_calls.into(),
        r.tool_calls_failed.into(),
        r.artifacts.into(),
        Cell::opt_decimal(r.p95_latency_ms),
        Cell::list(&r.models),
        Cell::list(&r.clients),
        r.judged.into(),
        Cell::opt_decimal(r.completion_avg),
        r.achieved.into(),
        Cell::opt_time(Some(r.first_used)),
        Cell::opt_time(Some(r.last_used)),
    ]
}

#[async_trait]
impl DataSet for Skills {
    fn id(&self) -> &'static str {
        "analysis-skills"
    }
    fn title(&self) -> &'static str {
        "Skills"
    }
    fn description(&self) -> &'static str {
        "One row per skill invoked in the window, with its conversations' figures."
    }
    fn columns(&self) -> &'static [Column] {
        SKILL_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    fn cap(&self) -> i64 {
        SKILL_CAP
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let picked: SkillIdsQuery = ctx.query()?;
        let rows = list_skill_facts(
            ctx.pool,
            &window(ctx).await?,
            &SkillListFilter {
                limit: ctx.limit,
                skills: picked.skills(),
                marketplace_id: trimmed(picked.marketplace.as_deref()).map(MarketplaceId::new),
                client_kind: trimmed(picked.client.as_deref()),
                search: trimmed(picked.search.as_deref())
                    .or_else(|| trimmed(picked.skill.as_deref())),
                ..SkillListFilter::default()
            },
        )
        .await?;
        let total = rows.first().map_or(0, |r| r.total);
        Ok(Table {
            rows: rows.iter().map(skill_row).collect(),
            total,
        })
    }
}
