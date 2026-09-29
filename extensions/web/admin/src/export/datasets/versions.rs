//! `/admin/analysis/versions` — every marketplace's rollup, and the version
//! history of one marketplace.

use async_trait::async_trait;
use serde::Deserialize;
use systemprompt::identifiers::MarketplaceId;

use crate::error::{AdminError, AdminResult};
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::marketplace_versions::may_read_marketplace;
use crate::repositories::analysis::marketplace_versions::{
    MarketplaceListRow, MarketplaceVersionMetricsRow, VersionWindow, list_marketplace_rollups,
    list_marketplace_version_metrics,
};
use crate::types::UserContext;

pub(crate) struct Marketplaces;
pub(crate) struct Versions;

#[derive(Debug, Default, Deserialize)]
struct Target {
    marketplace: Option<String>,
}

fn window(ctx: &ExportContext<'_>) -> AdminResult<VersionWindow> {
    let w = ctx.window()?;
    Ok(VersionWindow {
        start: w.from,
        end: w.to,
    })
}

const ROLLUP_COLUMNS: &[Column] = &[
    Column::new("marketplace_id", "Marketplace", CellKind::Text),
    Column::new("name", "Name", CellKind::Text),
    Column::new("content_hash", "Current version", CellKind::Text),
    Column::new("source", "Source", CellKind::Text).optional(),
    Column::new("source_hash", "Source hash", CellKind::Text).optional(),
    Column::new("plugins", "Plugins", CellKind::Integer),
    Column::new("skills", "Skills", CellKind::Integer),
    Column::new("versions", "Versions", CellKind::Integer),
    Column::new("first_seen_at", "First seen", CellKind::Timestamp),
    Column::new("last_changed_at", "Last changed", CellKind::Timestamp),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new(
        "failed_invocations",
        "Failed invocations",
        CellKind::Integer,
    ),
    Column::new("users", "Users", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("failed", "Failed requests", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
];

fn rollup_row(r: &MarketplaceListRow) -> Vec<Cell> {
    vec![
        r.marketplace_id.as_str().into(),
        Cell::opt_text(r.name.as_deref()),
        Cell::opt_text(r.content_hash.as_deref()),
        Cell::opt_text(r.source.as_deref()),
        Cell::opt_text(r.source_hash.as_deref()),
        Cell::opt_int(r.plugin_count),
        Cell::opt_int(r.skill_count),
        r.versions.into(),
        r.first_seen_at.into(),
        r.last_changed_at.into(),
        r.invocations.into(),
        r.failed_invocations.into(),
        r.users.into(),
        r.requests.into(),
        r.failed.into(),
        Cell::Money(r.cost),
    ]
}

const VERSION_COLUMNS: &[Column] = &[
    Column::new("marketplace_id", "Marketplace", CellKind::Text),
    Column::new("content_hash", "Version", CellKind::Text),
    Column::new("source", "Source", CellKind::Text),
    Column::new("source_hash", "Source hash", CellKind::Text).optional(),
    Column::new("origin", "Origin", CellKind::Text),
    Column::new("first_seen_at", "First seen", CellKind::Timestamp),
    Column::new("last_seen_at", "Last seen", CellKind::Timestamp),
    Column::new("effective_until", "Effective until", CellKind::Timestamp),
    Column::new("plugins", "Plugins", CellKind::Integer),
    Column::new("skills", "Skills", CellKind::Integer),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new(
        "failed_invocations",
        "Failed invocations",
        CellKind::Integer,
    ),
    Column::new("users", "Users", CellKind::Integer),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("failed", "Failed requests", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("p50_ms", "p50 (ms)", CellKind::Decimal),
    Column::new("p95_ms", "p95 (ms)", CellKind::Decimal),
];

fn version_row(r: &MarketplaceVersionMetricsRow) -> Vec<Cell> {
    vec![
        r.marketplace_id.as_str().into(),
        r.content_hash.as_str().into(),
        r.source.as_str().into(),
        Cell::opt_text(r.source_hash.as_deref()),
        r.origin.as_str().into(),
        r.first_seen_at.into(),
        r.last_seen_at.into(),
        Cell::opt_time(r.effective_until),
        r.plugin_count.into(),
        r.skill_count.into(),
        r.invocations.into(),
        r.failed_invocations.into(),
        r.users.into(),
        r.conversations.into(),
        r.requests.into(),
        r.failed.into(),
        r.tokens.into(),
        Cell::Money(r.cost),
        Cell::opt_decimal(r.p50_ms),
        Cell::opt_decimal(r.p95_ms),
    ]
}

#[async_trait]
impl DataSet for Marketplaces {
    fn id(&self) -> &'static str {
        "analysis-marketplaces"
    }
    fn title(&self) -> &'static str {
        "Marketplaces"
    }
    fn description(&self) -> &'static str {
        "One row per marketplace: current hash, contents, version count and usage over the window."
    }
    fn columns(&self) -> &'static [Column] {
        ROLLUP_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Days
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    // Why: a participant's export carries the marketplaces the page shows
    // them, filtered by the same rule.
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let rows = list_marketplace_rollups(ctx.pool, window(ctx)?).await?;
        Ok(Table::complete(
            rows.iter()
                .filter(|r| may_read_marketplace(ctx.user, &r.marketplace_id))
                .map(rollup_row)
                .collect(),
        ))
    }
}

#[async_trait]
impl DataSet for Versions {
    fn id(&self) -> &'static str {
        "analysis-versions"
    }
    fn title(&self) -> &'static str {
        "Marketplace versions"
    }
    fn description(&self) -> &'static str {
        "One row per version of a marketplace: when it was live, contents and usage while it was."
    }
    fn columns(&self) -> &'static [Column] {
        VERSION_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Days
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
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
        let rows = list_marketplace_version_metrics(ctx.pool, window(ctx)?, &id).await?;
        Ok(Table::complete(rows.iter().map(version_row).collect()))
    }
}
