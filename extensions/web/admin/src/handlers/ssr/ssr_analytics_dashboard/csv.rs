//! The Cost tab's export.
//!
//! Two audiences, two different files. The internal export carries provider
//! cost; the customer export carries consumption and no cost column at all,
//! because it is sent outside the platform team. The audience is chosen by the
//! same `?audience=` parameter the tab's toggle sets, so the file always
//! matches the view the operator was looking at when they clicked export.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::handlers::ssr::csv::{CsvBuilder, usd};
use crate::repositories::analytics::site::cost::{list_container_usage, list_provider_costs};
use crate::repositories::analytics::site::resolve_site_scope;
use crate::types::UserContext;

use super::{AnalyticsDashboardQuery, resolve_range};

pub(crate) async fn cost_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AnalyticsDashboardQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }
    let range = resolve_range(&query);
    let scope = resolve_site_scope(&pool, &query.scope(), query.attribution()).await?;

    if query.is_internal_audience() {
        let mut csv = CsvBuilder::new(&["provider", "requests", "tokens", "cost_usd"]);
        for row in list_provider_costs(&pool, range, &scope).await? {
            csv.row(&[
                &row.label,
                &row.requests.to_string(),
                &row.tokens.to_string(),
                &usd(row.cost_microdollars),
            ]);
        }
        return Ok(csv.into_response("analytics-cost-internal.csv"));
    }

    // Why: no cost column, by construction — `list_container_usage` selects
    // none, so this file cannot leak a supplier figure even if edited badly.
    let mut csv = CsvBuilder::new(&[
        "container",
        "users",
        "sessions",
        "requests",
        "input_tokens",
        "output_tokens",
    ]);
    for row in list_container_usage(&pool, range, &scope, query.container_axis()).await? {
        csv.row(&[
            &row.container_id,
            &row.users.to_string(),
            &row.sessions.to_string(),
            &row.requests.to_string(),
            &row.input_tokens.to_string(),
            &row.output_tokens.to_string(),
        ]);
    }
    Ok(csv.into_response("analytics-cost-customer.csv"))
}
