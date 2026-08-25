//! `/admin/reports/customer.csv` — the customer usage report as a download.
//!
//! Scoped exactly as the page one module up: a platform admin may name any
//! organization with `?org=`, everyone else exports their own. `?dimension=`
//! picks users (default), departments, or models.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::handlers::ssr::csv::CsvBuilder;
use crate::repositories::organizations::crud;
use crate::repositories::reports::customer;
use crate::types::UserContext;
use crate::util::month_range::{MonthQuery, parse_month_range};

#[derive(Debug, Deserialize)]
pub(crate) struct CustomerCsvQuery {
    pub month: Option<String>,
    pub org: Option<String>,
    pub dimension: Option<String>,
}

pub(crate) async fn report_customer_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<CustomerCsvQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }

    let month = parse_month_range(&MonthQuery {
        month: query.month.clone(),
    });
    let slug = super::resolve_slug(&pool, &user_ctx, query.org.as_deref()).await?;
    let Some(org) = crud::find_organization_by_slug(&pool, &slug).await? else {
        return Err(AdminError::NotFound(format!(
            "No organization with slug '{slug}'."
        )));
    };

    let dimension = query.dimension.as_deref().unwrap_or("users");
    let filename = format!("usage-{slug}-{}-{dimension}.csv", month.key);

    let csv = match dimension {
        "departments" => departments_csv(
            customer::list_customer_month_departments(&pool, &org.id, month.from, month.to).await?,
        ),
        "models" => models_csv(
            customer::list_customer_month_models(&pool, &org.id, month.from, month.to).await?,
        ),
        _ => users_csv(
            customer::list_customer_month_users(&pool, &org.id, month.from, month.to).await?,
        ),
    };
    Ok(csv.into_response(&filename))
}

fn users_csv(rows: Vec<customer::CustomerUserUsage>) -> CsvBuilder {
    let mut csv = CsvBuilder::new(&[
        "email",
        "display_name",
        "department",
        "requests",
        "input_tokens",
        "output_tokens",
        "distinct_models",
    ]);
    for r in rows {
        csv.row(&[
            &r.email,
            &r.display_name,
            &r.department,
            &r.requests.to_string(),
            &r.input_tokens.to_string(),
            &r.output_tokens.to_string(),
            &r.distinct_models.to_string(),
        ]);
    }
    csv
}

fn departments_csv(rows: Vec<customer::CustomerDepartmentUsage>) -> CsvBuilder {
    let mut csv = CsvBuilder::new(&[
        "department",
        "members",
        "requests",
        "input_tokens",
        "output_tokens",
    ]);
    for r in rows {
        csv.row(&[
            &r.department,
            &r.members.to_string(),
            &r.requests.to_string(),
            &r.input_tokens.to_string(),
            &r.output_tokens.to_string(),
        ]);
    }
    csv
}

fn models_csv(rows: Vec<customer::CustomerModelUsage>) -> CsvBuilder {
    let mut csv = CsvBuilder::new(&[
        "provider",
        "model",
        "requests",
        "input_tokens",
        "output_tokens",
        "cache_read_tokens",
    ]);
    for r in rows {
        csv.row(&[
            &r.provider,
            &r.model,
            &r.requests.to_string(),
            &r.input_tokens.to_string(),
            &r.output_tokens.to_string(),
            &r.cache_read_tokens.to_string(),
        ]);
    }
    csv
}
