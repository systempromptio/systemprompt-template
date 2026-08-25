//! `/admin/reports/internal.csv` — the month-end P&L as a download.
//!
//! Same queries and month resolution as the page one module up, so the
//! spreadsheet can never disagree with the screen. `?dimension=` picks the
//! slice — organization (default), provider, or model — because a finance
//! import wants one flat table, not three stacked ones.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::handlers::ssr::csv::{CsvBuilder, usd};
use crate::repositories::reports::internal;
use crate::util::month_range::{MonthQuery, parse_month_range};

#[derive(Debug, Deserialize)]
pub(crate) struct InternalCsvQuery {
    pub month: Option<String>,
    pub dimension: Option<String>,
}

pub(crate) async fn report_internal_csv(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<InternalCsvQuery>,
) -> AdminResult<Response> {
    let month = parse_month_range(&MonthQuery {
        month: query.month.clone(),
    });
    let dimension = query.dimension.as_deref().unwrap_or("organization");
    let filename = format!("pnl-{}-{dimension}.csv", month.key);

    let csv = match dimension {
        "provider" => {
            supplier_csv(internal::list_provider_month_costs(&pool, month.from, month.to).await?)
        },
        "model" => {
            supplier_csv(internal::list_model_month_costs(&pool, month.from, month.to).await?)
        },
        _ => organization_csv(
            internal::list_organization_month_pnl(&pool, month.from, month.to).await?,
        ),
    };
    Ok(csv.into_response(&filename))
}

fn organization_csv(rows: Vec<internal::OrganizationMonthPnl>) -> CsvBuilder {
    let mut csv = CsvBuilder::new(&[
        "slug",
        "name",
        "plan",
        "revenue_usd",
        "cost_usd",
        "margin_usd",
        "requests",
        "tokens",
        "seats_used",
        "active_users",
    ]);
    for r in rows {
        csv.row(&[
            &r.slug,
            &r.name,
            r.plan_name.as_deref().unwrap_or(""),
            &usd(r.revenue_microdollars),
            &usd(r.cost_microdollars),
            &usd(r.margin_microdollars()),
            &r.requests.to_string(),
            &r.tokens.to_string(),
            &r.seats_used.to_string(),
            &r.active_users.to_string(),
        ]);
    }
    csv
}

fn supplier_csv(rows: Vec<internal::SupplierMonthCost>) -> CsvBuilder {
    let mut csv = CsvBuilder::new(&["key", "requests", "tokens", "cost_usd"]);
    for r in rows {
        csv.row(&[
            &r.key,
            &r.requests.to_string(),
            &r.tokens.to_string(),
            &usd(r.cost_microdollars),
        ]);
    }
    csv
}
