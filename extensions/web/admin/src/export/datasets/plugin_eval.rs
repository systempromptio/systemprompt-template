//! `/admin/analysis/versions/{marketplace}?tab=evaluation` — every
//! conversation that invoked one of a marketplace's skills, scored by the
//! fixed rules in `plugin_eval.sql`. The Evaluation tab aggregates exactly
//! these rows, and the eval harness downloads them with a PAT, so the page,
//! the file and a run's report are one set of numbers.

use async_trait::async_trait;
use serde::Deserialize;
use systemprompt::identifiers::MarketplaceId;

use crate::error::{AdminError, AdminResult};
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::marketplace_versions::may_read_marketplace;
use crate::repositories::analysis::marketplace_versions::VersionWindow;
use crate::repositories::analysis::plugin_eval::{
    PluginEvalRow, PluginEvalToolRow, list_plugin_eval_runs, list_plugin_eval_tools,
};
use crate::types::UserContext;

pub(crate) struct PluginEval;
pub(crate) struct PluginEvalTools;

#[derive(Debug, Default, Deserialize)]
struct Target {
    marketplace: Option<String>,
}

const COLUMNS: &[Column] = &[
    Column::new("context_id", "Conversation", CellKind::Text),
    Column::new("client_session_id", "Client session", CellKind::Text),
    Column::new("plugin_id", "Plugin", CellKind::Text),
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("marketplace_hash", "Version", CellKind::Text),
    Column::new("first_invoked_at", "Invoked", CellKind::Timestamp),
    Column::new("success", "Success", CellKind::Bool),
    Column::new("completed", "Completed", CellKind::Bool),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_read_tokens", "Cache read tokens", CellKind::Integer),
    Column::new(
        "cache_creation_tokens",
        "Cache creation tokens",
        CellKind::Integer,
    ),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("duration_seconds", "Duration (s)", CellKind::Integer),
    Column::new("p50_ms", "p50 (ms)", CellKind::Integer),
    Column::new("p95_ms", "p95 (ms)", CellKind::Integer),
    Column::new("mcp_calls", "MCP calls", CellKind::Integer),
    Column::new(
        "connector_calls",
        "Plugin connector calls",
        CellKind::Integer,
    ),
    Column::new("builtin_calls", "Built-in tool calls", CellKind::Integer),
    Column::new("failed_calls", "Failed calls", CellKind::Integer),
    Column::new("schema_errors", "Schema errors", CellKind::Integer),
    Column::new("access_errors", "Access errors", CellKind::Integer),
    Column::new("upstream_errors", "Upstream errors", CellKind::Integer),
    Column::new("bad_arguments", "Bad arguments", CellKind::Integer),
    Column::new("timeouts", "Timeouts", CellKind::Integer),
    Column::new("repeated_calls", "Repeated calls", CellKind::Integer),
    Column::new("widgets", "Widgets rendered", CellKind::Integer),
    Column::new("writes", "Writes", CellKind::Integer),
    Column::new("answer_chars", "Answer length", CellKind::Integer),
    Column::new(
        "placeholder_mentions",
        "Placeholder mentions",
        CellKind::Integer,
    ),
    Column::new("tools_unavailable", "Tools unavailable", CellKind::Bool),
];

fn row(r: &PluginEvalRow) -> Vec<Cell> {
    vec![
        r.context_id.as_str().into(),
        Cell::opt_text(r.client_session_id.as_deref()),
        r.plugin_id.as_str().into(),
        r.skill.as_str().into(),
        r.marketplace_hash.as_str().into(),
        r.first_invoked_at.into(),
        r.success.into(),
        r.completed.into(),
        r.turns.into(),
        r.requests.into(),
        r.input_tokens.into(),
        r.output_tokens.into(),
        r.cache_read_tokens.into(),
        r.cache_creation_tokens.into(),
        Cell::Money(r.cost),
        Cell::opt_int(r.duration_seconds),
        Cell::opt_int(r.p50_ms),
        Cell::opt_int(r.p95_ms),
        r.mcp_calls.into(),
        r.connector_calls.into(),
        r.builtin_calls.into(),
        r.failed_calls.into(),
        r.schema_errors.into(),
        r.access_errors.into(),
        r.upstream_errors.into(),
        r.bad_arguments.into(),
        r.timeouts.into(),
        r.repeated_calls.into(),
        r.widgets.into(),
        r.writes.into(),
        r.answer_chars.into(),
        r.placeholder_mentions.into(),
        r.tools_unavailable.into(),
    ]
}

#[async_trait]
impl DataSet for PluginEval {
    fn id(&self) -> &'static str {
        "analysis-plugin-eval"
    }
    fn title(&self) -> &'static str {
        "Plugin evaluation"
    }
    fn description(&self) -> &'static str {
        "One row per conversation that invoked a marketplace's skill: version, cost, tokens, tool calls, error classes and the fixed-rule success."
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Days
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (id, window) = target(ctx)?;
        let rows = list_plugin_eval_runs(ctx.pool, window, &id).await?;
        Ok(Table::complete(rows.iter().map(row).collect()))
    }
}

fn target(ctx: &ExportContext<'_>) -> AdminResult<(MarketplaceId, VersionWindow)> {
    let target: Target = ctx.query()?;
    let id = target
        .marketplace
        .filter(|m| !m.is_empty() && m.len() <= 128 && !m.chars().any(char::is_control))
        .ok_or_else(|| AdminError::BadRequest("Name a marketplace".to_owned()))?;
    let id = MarketplaceId::new(id);
    // Why: the same not-found the page answers, so the export is no
    // oracle for marketplaces outside the caller's participation.
    if !may_read_marketplace(ctx.user, &id) {
        return Err(AdminError::NotFound(format!(
            "No version of marketplace '{id}' has been recorded"
        )));
    }
    let w = ctx.window()?;
    Ok((
        id,
        VersionWindow {
            start: w.from,
            end: w.to,
        },
    ))
}

const TOOL_COLUMNS: &[Column] = &[
    Column::new("marketplace_hash", "Version", CellKind::Text),
    Column::new("plugin_id", "Plugin", CellKind::Text),
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("tool", "Tool", CellKind::Text),
    Column::new("calls", "Calls", CellKind::Integer),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("failed_calls", "Failed calls", CellKind::Integer),
    Column::new("schema_errors", "Schema errors", CellKind::Integer),
    Column::new("access_errors", "Access errors", CellKind::Integer),
    Column::new("upstream_errors", "Upstream errors", CellKind::Integer),
    Column::new("bad_arguments", "Bad arguments", CellKind::Integer),
    Column::new("timeouts", "Timeouts", CellKind::Integer),
    Column::new("repeated_calls", "Repeated calls", CellKind::Integer),
];

fn tool_row(r: &PluginEvalToolRow) -> Vec<Cell> {
    vec![
        r.marketplace_hash.as_str().into(),
        r.plugin_id.as_str().into(),
        r.skill.as_str().into(),
        r.tool.as_str().into(),
        r.calls.into(),
        r.conversations.into(),
        r.failed_calls.into(),
        r.schema_errors.into(),
        r.access_errors.into(),
        r.upstream_errors.into(),
        r.bad_arguments.into(),
        r.timeouts.into(),
        r.repeated_calls.into(),
    ]
}

#[async_trait]
impl DataSet for PluginEvalTools {
    fn id(&self) -> &'static str {
        "analysis-plugin-eval-tools"
    }
    fn title(&self) -> &'static str {
        "Plugin evaluation by tool"
    }
    fn description(&self) -> &'static str {
        "One row per version, skill and tool: calls, failed calls by class and repeated calls, by the same rules as analysis-plugin-eval."
    }
    fn columns(&self) -> &'static [Column] {
        TOOL_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Days
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (id, window) = target(ctx)?;
        let rows = list_plugin_eval_tools(ctx.pool, window, &id).await?;
        Ok(Table::complete(rows.iter().map(tool_row).collect()))
    }
}
