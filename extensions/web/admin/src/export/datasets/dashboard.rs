//! `/admin/analytics` — the tables behind the dashboard's Models and Sessions
//! tabs; the Skills and Tools tabs' tables are `dashboard_tools`, the Cost
//! tab's `dashboard_cost`.

use async_trait::async_trait;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::export::window::time_range;
use crate::handlers::ssr::ssr_analytics_dashboard::AnalyticsDashboardQuery;
use crate::repositories::analytics::site::models::list_model_stats;
use crate::repositories::analytics::site::sessions::list_session_costs_paged;
use crate::repositories::analytics::site::{SiteScope, resolve_site_scope};
use crate::util::time_range::TimeRange;

pub(crate) struct ModelStats;
pub(crate) struct SessionCosts;

pub(super) struct DashboardRead {
    pub(super) query: AnalyticsDashboardQuery,
    pub(super) scope: SiteScope,
    pub(super) range: TimeRange,
}

pub(super) async fn plan(ctx: &ExportContext<'_>) -> AdminResult<DashboardRead> {
    let query: AnalyticsDashboardQuery = ctx.query()?;
    let scope = resolve_site_scope(ctx.pool, &query.scope(), query.attribution()).await?;
    Ok(DashboardRead {
        query,
        scope,
        range: time_range(ctx)?,
    })
}

// Why: the five tab tables and the three cost tables differ only in their
// columns and the read behind them; the trait boilerplate is one macro so
// each table is its column list and its query.
macro_rules! live_dataset {
    ($ty:ident, $id:literal, $title:literal, $description:literal, $columns:ident,
     |$ctx:ident| $load:expr) => {
        #[async_trait]
        impl DataSet for $ty {
            fn id(&self) -> &'static str {
                $id
            }
            fn title(&self) -> &'static str {
                $title
            }
            fn description(&self) -> &'static str {
                $description
            }
            fn columns(&self) -> &'static [Column] {
                $columns
            }
            fn window(&self) -> Window {
                Window::Live
            }
            async fn load(&self, $ctx: &ExportContext<'_>) -> AdminResult<Table> {
                $load
            }
        }
    };
}
pub(super) use live_dataset;

const MODEL_COLUMNS: &[Column] = &[
    Column::new("model", "Model", CellKind::Text),
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_tokens", "Cache tokens", CellKind::Integer),
    Column::new("reasoning_tokens", "Reasoning tokens", CellKind::Integer).optional(),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("p50_latency_ms", "p50 latency (ms)", CellKind::Decimal),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Decimal),
    Column::new("errors", "Errors", CellKind::Integer),
    Column::new("redirected", "Redirected", CellKind::Integer).optional(),
    Column::new("unrouted", "Unrouted", CellKind::Bool).optional(),
];

live_dataset!(
    ModelStats,
    "analytics-models",
    "Models",
    "One row per model: requests, tokens, cost, latency and errors over the window.",
    MODEL_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let rows = list_model_stats(ctx.pool, p.range, &p.scope).await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.model.as_str().into(),
                        Cell::opt_text(r.provider.as_deref()),
                        r.requests.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                        r.cache_tokens.into(),
                        r.reasoning_tokens.into(),
                        Cell::Money(r.cost_microdollars),
                        Cell::opt_decimal(r.p50_latency_ms),
                        Cell::opt_decimal(r.p95_latency_ms),
                        r.errors.into(),
                        r.redirected.into(),
                        r.is_unrouted.into(),
                    ]
                })
                .collect(),
        ))
    }
);

// Why: the Sessions tab reads client-reported statusline snapshots, one per
// session; the export is the same rows, labelled as the client's own totals.
const SESSION_COLUMNS: &[Column] = &[
    Column::new("updated_at", "Updated", CellKind::Timestamp),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("cost_usd", "Cost (USD, client-reported)", CellKind::Money),
    Column::new("context_window", "Context window", CellKind::Integer).optional(),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_read_tokens", "Cache read tokens", CellKind::Integer),
    Column::new("rating", "Rating", CellKind::Integer).optional(),
    Column::new("outcome", "Outcome", CellKind::Text).optional(),
];

live_dataset!(
    SessionCosts,
    "analytics-sessions",
    "Session costs",
    "One row per client session: person, model, tokens and the client's own cost total over the window.",
    SESSION_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let (rows, total) =
            list_session_costs_paged(ctx.pool, p.range, &p.scope, ctx.limit, 0).await?;
        Ok(Table {
            rows: rows
                .iter()
                .map(|r| {
                    vec![
                        r.updated_at.into(),
                        r.session_id.as_str().into(),
                        r.user_id.as_str().into(),
                        Cell::opt_text(r.model.as_deref()),
                        Cell::Money(r.total_cost_microdollars),
                        r.context_window_size.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                        r.cache_read_tokens.into(),
                        Cell::opt_int(r.rating),
                        Cell::opt_text(r.outcome.as_deref()),
                    ]
                })
                .collect(),
            total,
        })
    }
);
