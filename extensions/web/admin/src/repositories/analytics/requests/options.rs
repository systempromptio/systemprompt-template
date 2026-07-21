//! Distinct model / provider / status values for the requests-page dropdowns.
//!
//! Aggregated over the same window the page renders so the dropdowns only show
//! options that appear in the current view.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance::time_range::TimeRange;

#[derive(Debug, Clone, Default, Serialize)]
pub struct RequestFilterOptions {
    pub models: Vec<String>,
    pub providers: Vec<String>,
    pub statuses: Vec<String>,
}

pub async fn fetch_request_filter_options(
    pool: &PgPool,
    range: TimeRange,
) -> Result<RequestFilterOptions, sqlx::Error> {
    let models = sqlx::query_scalar!(
        r#"SELECT DISTINCT model AS "model!"
           FROM ai_requests
           WHERE created_at >= $1 AND created_at < $2
             AND model IS NOT NULL AND model <> ''
           ORDER BY model"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    let providers = sqlx::query_scalar!(
        r#"SELECT DISTINCT provider AS "provider!"
           FROM ai_requests
           WHERE created_at >= $1 AND created_at < $2
             AND provider IS NOT NULL AND provider <> ''
           ORDER BY provider"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    let statuses = sqlx::query_scalar!(
        r#"SELECT DISTINCT status AS "status!"
           FROM ai_requests
           WHERE created_at >= $1 AND created_at < $2
             AND status IS NOT NULL AND status <> ''
           ORDER BY status"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    Ok(RequestFilterOptions {
        models,
        providers,
        statuses,
    })
}
