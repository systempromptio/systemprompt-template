//! `/admin/analytics?tab=cost` — provider cost, cost by day and, for the
//! customer audience, consumption by container.

use async_trait::async_trait;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories::analytics::site::cost::{
    list_container_usage, list_provider_cost_by_day, list_provider_costs,
};

use super::dashboard::{live_dataset, plan};

pub(crate) struct ProviderCosts;
pub(crate) struct ProviderCostByDay;
pub(crate) struct ContainerUsage;

const SUPPLIER_COLUMNS: &[Column] = &[
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
];

live_dataset!(
    ProviderCosts,
    "analytics-cost-providers",
    "Provider cost",
    "One row per provider: requests, tokens and cost over the window.",
    SUPPLIER_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let rows = list_provider_costs(ctx.pool, p.range, &p.scope).await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.label.as_str().into(),
                        r.requests.into(),
                        r.tokens.into(),
                        Cell::Money(r.cost_microdollars),
                    ]
                })
                .collect(),
        ))
    }
);

const DAY_COLUMNS: &[Column] = &[
    Column::new("day", "Day", CellKind::Text),
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
];

live_dataset!(
    ProviderCostByDay,
    "analytics-cost-days",
    "Provider cost by day",
    "One row per day and provider: requests and cost.",
    DAY_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let rows = list_provider_cost_by_day(ctx.pool, p.range, &p.scope).await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.day.to_string().into(),
                        r.provider.as_str().into(),
                        r.requests.into(),
                        Cell::Money(r.cost_microdollars),
                    ]
                })
                .collect(),
        ))
    }
);

// Why: no cost column, by construction — `list_container_usage` selects
// none, so this table is the one that can be sent outside the platform team.
const CONTAINER_COLUMNS: &[Column] = &[
    Column::new("container", "Container", CellKind::Text),
    Column::new("users", "Users", CellKind::Integer),
    Column::new("sessions", "Sessions", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
];

live_dataset!(
    ContainerUsage,
    "analytics-cost-containers",
    "Consumption by container",
    "One row per container: people, sessions, requests and tokens. No cost.",
    CONTAINER_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let rows =
            list_container_usage(ctx.pool, p.range, &p.scope, p.query.container_axis()).await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.container_id.as_str().into(),
                        r.users.into(),
                        r.sessions.into(),
                        r.requests.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                    ]
                })
                .collect(),
        ))
    }
);
