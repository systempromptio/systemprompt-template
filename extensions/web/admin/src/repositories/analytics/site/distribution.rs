//! Model-usage distribution feeding the pie chart.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Clone)]
pub struct ModelDistributionRow {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

pub async fn list_model_distribution(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<ModelDistributionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            r.model AS "model!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens + r.output_tokens), 0)::BIGINT AS "tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
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
        ORDER BY COUNT(*) DESC
        "#,
        range.from,
        range.to,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
        scope.user_id_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ModelDistributionRow {
            model: r.model,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
        })
        .collect())
}
