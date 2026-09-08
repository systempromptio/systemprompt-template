//! `/admin/reports/internal.csv` — the provider cost report as a download.
//!
//! Same queries and month resolution as the page one module up, so the
//! spreadsheet can never disagree with the screen. `?dimension=` picks the
//! slice — provider (default) or model — because a finance
//! import wants one flat table, not three stacked ones.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::handlers::ssr::csv::{CsvBuilder, usd};
use crate::repositories::dashboard_reports::internal;
use crate::types::UserContext;
use crate::util::month_range::{MonthQuery, parse_month_range};

#[derive(Debug, Deserialize)]
pub(crate) struct InternalCsvQuery {
    pub month: Option<String>,
    pub dimension: Option<String>,
}

pub(crate) async fn report_internal_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<InternalCsvQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }
    let month = parse_month_range(&MonthQuery {
        month: query.month.clone(),
    });
    let dimension = query.dimension.as_deref().unwrap_or("provider");
    let filename = format!("provider-cost-{}-{dimension}.csv", month.key);

    let csv = match dimension {
        "model" => {
            supplier_csv(internal::list_model_month_costs(&pool, month.from, month.to).await?)
        },
        _ => supplier_csv(internal::list_provider_month_costs(&pool, month.from, month.to).await?),
    };
    Ok(csv.into_response(&filename))
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
