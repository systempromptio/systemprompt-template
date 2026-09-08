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
            COALESCE(r.model, 'unrouted') AS "model!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(COALESCE(r.tokens_used,
                  COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0)
                + COALESCE(r.cache_read_tokens, 0) + COALESCE(r.cache_creation_tokens, 0))), 0)::BIGINT AS "tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        GROUP BY COALESCE(r.model, 'unrouted')
        ORDER BY COUNT(*) DESC
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
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
