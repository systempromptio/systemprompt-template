//! Day/week-bucketed usage series feeding the volume and cost trend charts.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

/// `date_trunc` unit, constrained to the two the page offers so the bind can
/// never smuggle an arbitrary unit into the query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeriesBucket {
    #[default]
    Day,
    Week,
}

impl SeriesBucket {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
        }
    }

    #[must_use]
    pub fn from_bucket_param(raw: Option<&str>) -> Self {
        match raw {
            Some("week") => Self::Week,
            _ => Self::Day,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UsageBucket {
    pub bucket_start: chrono::DateTime<chrono::Utc>,
    pub requests: i64,
    pub errors: i64,
    pub cost_microdollars: i64,
    pub active_users: i64,
}

// Why: Zero-filled calendar spine `LEFT JOIN`ed to the aggregate, so quiet days
// render as gaps rather than disappearing and compressing the x-axis.
pub async fn list_daily_usage_series(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    bucket: SeriesBucket,
) -> Result<Vec<UsageBucket>, sqlx::Error> {
    let unit = bucket.as_str();
    let rows = sqlx::query!(
        r#"
        WITH spine AS (
            SELECT generate_series(
                DATE_TRUNC($5, $1::TIMESTAMPTZ),
                DATE_TRUNC($5, $2::TIMESTAMPTZ),
                ('1 ' || $5)::INTERVAL
            ) AS bucket_start
        ),
        agg AS (
            SELECT
                DATE_TRUNC($5, r.created_at) AS bucket_start,
                COUNT(*)::BIGINT AS requests,
                COUNT(*) FILTER (WHERE r.status NOT IN ('completed', 'success', 'pending', 'streaming'))::BIGINT
                    AS errors,
                COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost,
                COUNT(DISTINCT r.user_id)::BIGINT AS active_users
            FROM ai_requests r
            WHERE r.created_at >= $1 AND r.created_at < $2
              AND NOT r.synthetic
              AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
              AND ($4::TEXT IS NULL OR r.user_id = $4)
            GROUP BY 1
        )
        SELECT
            s.bucket_start AS "bucket_start!",
            COALESCE(a.requests, 0)::BIGINT AS "requests!",
            COALESCE(a.errors, 0)::BIGINT AS "errors!",
            COALESCE(a.cost, 0)::BIGINT AS "cost!",
            COALESCE(a.active_users, 0)::BIGINT AS "active_users!"
        FROM spine s
        LEFT JOIN agg a ON a.bucket_start = s.bucket_start
        ORDER BY s.bucket_start
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        unit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UsageBucket {
            bucket_start: r.bucket_start,
            requests: r.requests,
            errors: r.errors,
            cost_microdollars: r.cost,
            active_users: r.active_users,
        })
        .collect())
}
