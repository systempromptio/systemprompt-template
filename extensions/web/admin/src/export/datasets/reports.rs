//! The month-end reports — usage for a customer hand-off, provider cost for
//! finance. Month-keyed (`?month=YYYY-MM`) rather than windowed, because a
//! finance import wants calendar months.
//!
//! The customer tables select no cost column anywhere: the report leaves the
//! platform team, so "no internal figure leaks" is a property of the SQL
//! rather than a discipline a renderer has to keep.

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories::reports::{customer, internal};
use crate::repositories::scope::{ScopeRequest, SubjectScope};
use crate::util::month_range::{MonthQuery, MonthRange, parse_month_range};

pub(crate) struct CustomerUsers;
pub(crate) struct CustomerProjects;
pub(crate) struct CustomerModels;
pub(crate) struct InternalProviders;
pub(crate) struct InternalModels;

#[derive(Debug, Default, Deserialize)]
struct ReportQuery {
    month: Option<String>,
    group: Option<String>,
    project: Option<String>,
}

struct Scoped {
    month: MonthRange,
    scope: SubjectScope,
}

async fn scoped(ctx: &ExportContext<'_>) -> AdminResult<Scoped> {
    let query: ReportQuery = ctx.query()?;
    let request =
        ScopeRequest::from_query(ctx.user, query.group.as_deref(), query.project.as_deref());
    let scope =
        crate::repositories::scope::membership::get_subject_scope(ctx.pool, &request).await?;
    Ok(Scoped {
        month: parse_month_range(&MonthQuery { month: query.month }),
        scope,
    })
}

fn month(ctx: &ExportContext<'_>) -> AdminResult<MonthRange> {
    let query: ReportQuery = ctx.query()?;
    Ok(parse_month_range(&MonthQuery { month: query.month }))
}

macro_rules! dataset {
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
            // Why: the dialog's month picker writes the same `?month=` the
            // loaders parse, so declaring the window changes the control and
            // nothing the files contain.
            fn window(&self) -> Window {
                Window::Month
            }
            async fn load(&self, $ctx: &ExportContext<'_>) -> AdminResult<Table> {
                $load
            }
        }
    };
}

const USER_COLUMNS: &[Column] = &[
    Column::new("email", "Email", CellKind::Text),
    Column::new("display_name", "Name", CellKind::Text),
    Column::new("project", "Project", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("reasoning_tokens", "Reasoning tokens", CellKind::Integer),
    Column::new("total_tokens", "Total tokens", CellKind::Integer),
    Column::new("distinct_models", "Models", CellKind::Integer),
];

dataset!(
    CustomerUsers,
    "report-customer-users",
    "Customer usage by person",
    "One row per person active in the month: project, requests and tokens. No cost.",
    USER_COLUMNS,
    |ctx| {
        let s = scoped(ctx).await?;
        let rows =
            customer::list_customer_month_users(ctx.pool, &s.scope, s.month.from, s.month.to)
                .await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.email.as_str().into(),
                        r.display_name.as_str().into(),
                        Cell::opt_text(r.project.as_deref()),
                        r.requests.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                        r.reasoning_tokens.into(),
                        r.total_tokens.into(),
                        r.distinct_models.into(),
                    ]
                })
                .collect(),
        ))
    }
);

const PROJECT_COLUMNS: &[Column] = &[
    Column::new("project", "Project", CellKind::Text),
    Column::new("members", "Members", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("reasoning_tokens", "Reasoning tokens", CellKind::Integer),
    Column::new("total_tokens", "Total tokens", CellKind::Integer),
];

dataset!(
    CustomerProjects,
    "report-customer-projects",
    "Customer usage by project",
    "One row per project for the month: members, requests and tokens. No cost.",
    PROJECT_COLUMNS,
    |ctx| {
        let s = scoped(ctx).await?;
        let rows =
            customer::list_customer_month_projects(ctx.pool, &s.scope, s.month.from, s.month.to)
                .await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.project.as_str().into(),
                        r.members.into(),
                        r.requests.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                        r.reasoning_tokens.into(),
                        r.total_tokens.into(),
                    ]
                })
                .collect(),
        ))
    }
);

const MODEL_COLUMNS: &[Column] = &[
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_read_tokens", "Cache read tokens", CellKind::Integer),
    Column::new("reasoning_tokens", "Reasoning tokens", CellKind::Integer),
    Column::new("total_tokens", "Total tokens", CellKind::Integer),
];

dataset!(
    CustomerModels,
    "report-customer-models",
    "Customer usage by model",
    "One row per provider and model for the month: requests and tokens. No cost.",
    MODEL_COLUMNS,
    |ctx| {
        let s = scoped(ctx).await?;
        let rows =
            customer::list_customer_month_models(ctx.pool, &s.scope, s.month.from, s.month.to)
                .await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.provider.as_str().into(),
                        r.model.as_str().into(),
                        r.requests.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                        r.cache_read_tokens.into(),
                        r.reasoning_tokens.into(),
                        r.total_tokens.into(),
                    ]
                })
                .collect(),
        ))
    }
);

const SUPPLIER_COLUMNS: &[Column] = &[
    Column::new("key", "Supplier", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
];

fn supplier_rows(rows: &[internal::SupplierMonthCost]) -> Table {
    Table::complete(
        rows.iter()
            .map(|r| {
                vec![
                    r.key.as_str().into(),
                    r.requests.into(),
                    r.tokens.into(),
                    Cell::Money(r.cost_microdollars),
                ]
            })
            .collect(),
    )
}

dataset!(
    InternalProviders,
    "report-internal-providers",
    "Provider cost for the month",
    "One row per provider for the month: requests, tokens and cost.",
    SUPPLIER_COLUMNS,
    |ctx| {
        let m = month(ctx)?;
        Ok(supplier_rows(
            &internal::list_provider_month_costs(ctx.pool, m.from, m.to).await?,
        ))
    }
);

dataset!(
    InternalModels,
    "report-internal-models",
    "Model cost for the month",
    "One row per model for the month: requests, tokens and cost.",
    SUPPLIER_COLUMNS,
    |ctx| {
        let m = month(ctx)?;
        Ok(supplier_rows(
            &internal::list_model_month_costs(ctx.pool, m.from, m.to).await?,
        ))
    }
);
