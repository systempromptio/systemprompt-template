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

/// Zero-filled calendar spine `LEFT JOIN`ed to the aggregate, so quiet days
/// render as gaps rather than disappearing and compressing the x-axis.
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
                DATE_TRUNC($6, $1::TIMESTAMPTZ),
                DATE_TRUNC($6, $2::TIMESTAMPTZ),
                ('1 ' || $6)::INTERVAL
            ) AS bucket_start
        ),
        agg AS (
            SELECT
                DATE_TRUNC($6, r.created_at) AS bucket_start,
                COUNT(*)::BIGINT AS requests,
                COUNT(*) FILTER (WHERE r.status NOT IN ('completed', 'pending', 'streaming'))::BIGINT
                    AS errors,
                COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost,
                COUNT(DISTINCT r.user_id)::BIGINT AS active_users
            FROM ai_requests r
            LEFT JOIN organization_members m ON m.user_id = r.user_id
            LEFT JOIN organizations o ON o.id = m.org_id
            LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
            WHERE r.created_at >= $1 AND r.created_at < $2
              AND NOT r.synthetic
              AND ($3::TEXT IS NULL OR o.slug = $3)
              AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
              AND ($5::TEXT IS NULL OR r.user_id = $5)
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
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
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
