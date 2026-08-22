//! Cost-by-model over time, feeding the stacked bar beside the model pie.
//!
//! The top six models by window cost keep their own series; the tail folds
//! into "Other", matching the pie's fold so the seven chart color tokens
//! always suffice and model→color agrees between the two charts. Rows come
//! back sparse (no zero-filling here) — the view pivots them onto the same
//! bucket spine as the fetched usage series, which is authoritative.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;
use super::series::SeriesBucket;

#[derive(Debug, Clone)]
pub struct ModelCostBucket {
    pub bucket_start: chrono::DateTime<chrono::Utc>,
    pub model: String,
    pub cost_microdollars: i64,
    pub requests: i64,
}

pub async fn list_model_cost_series(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    bucket: SeriesBucket,
) -> Result<Vec<ModelCostBucket>, sqlx::Error> {
    let unit = bucket.as_str();
    let rows = sqlx::query!(
        r#"
        WITH ranked AS (
            SELECT r.model
            FROM ai_requests r
            LEFT JOIN organization_members m ON m.user_id = r.user_id
            LEFT JOIN organizations o ON o.id = m.org_id
            LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
            WHERE r.created_at >= $1 AND r.created_at < $2
              AND NOT r.synthetic
              AND ($3::TEXT IS NULL OR o.slug = $3)
              AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
              AND ($5::TEXT IS NULL OR r.user_id = $5)
            GROUP BY r.model
            ORDER BY SUM(r.cost_microdollars) DESC NULLS LAST
            LIMIT 6
        )
        SELECT
            DATE_TRUNC($6, r.created_at) AS "bucket_start!",
            CASE WHEN k.model IS NULL THEN 'Other' ELSE r.model END AS "model!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!",
            COUNT(*)::BIGINT AS "requests!"
        FROM ai_requests r
        LEFT JOIN ranked k ON k.model = r.model
        LEFT JOIN organization_members m ON m.user_id = r.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR r.user_id = $5)
        GROUP BY 1, 2
        ORDER BY 1, 2
        "#,
        range.from,
        range.to,
        scope.org_slug.as_slug(),
        scope.department.as_deref(),
        scope.user_id_str(),
        unit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ModelCostBucket {
            bucket_start: r.bucket_start,
            model: r.model,
            cost_microdollars: r.cost,
            requests: r.requests,
        })
        .collect())
}
