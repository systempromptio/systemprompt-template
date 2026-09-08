//! Cost and consumption for the Cost tab, in two audiences.
//!
//! The internal half reads provider cost — what inference cost us at the
//! supplier — by day, by provider, and by model. The customer half reads
//! `admin_usage_daily_rollups`, which carries the `group_id` and `project_id`
//! a person held when the row was rolled up, and selects no cost column
//! anywhere: the export is sent outside the platform team, so "no supplier
//! figure leaks" is a property of the SQL rather than a discipline the
//! template has to keep.
//!
//! Rollups keep historical attribution rather than today's membership, which
//! is why a container's cost history does not move when someone changes team.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Clone)]
pub struct CostDayRow {
    pub day: chrono::NaiveDate,
    pub provider: String,
    pub requests: i64,
    pub cost_microdollars: i64,
}

pub async fn list_provider_cost_by_day(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<CostDayRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            (r.created_at AT TIME ZONE 'UTC')::DATE AS "day!",
            COALESCE(r.provider, 'unrouted') AS "provider!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        GROUP BY 1, 2
        ORDER BY 1, 2
        LIMIT 2000
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
        .map(|r| CostDayRow {
            day: r.day,
            provider: r.provider,
            requests: r.requests,
            cost_microdollars: r.cost,
        })
        .collect())
}

/// A supplier line: one provider or one model, with what it cost and what it
/// served. Both halves of the internal report have the same shape, so they
/// share one row type and one renderer.
#[derive(Debug, Clone)]
pub struct SupplierCostRow {
    pub label: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

pub async fn list_provider_costs(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<SupplierCostRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(r.provider, 'unrouted') AS "label!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0)), 0)::BIGINT
                AS "tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        GROUP BY 1
        ORDER BY SUM(r.cost_microdollars) DESC NULLS LAST
        LIMIT 50
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
        .map(|r| SupplierCostRow {
            label: r.label,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
        })
        .collect())
}

/// What each container consumed, with no cost column at all. Read from the
/// rollups because they carry the attribution the container had at the time.
#[derive(Debug, Clone)]
pub struct ContainerUsageRow {
    pub container_id: String,
    pub users: i64,
    pub sessions: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

/// Which rollup column a container listing groups on. `Group` and `Project`
/// are the two the rollup stores; nothing else is attributable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerAxis {
    Group,
    Project,
}

impl ContainerAxis {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::Project => "project",
        }
    }
}

pub async fn list_container_usage(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    axis: ContainerAxis,
) -> Result<Vec<ContainerUsageRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(
                CASE WHEN $5::TEXT = 'group' THEN u.group_id ELSE u.project_id END,
                'unattributed'
            ) AS "container_id!",
            COUNT(DISTINCT u.user_id)::BIGINT AS "users!",
            COALESCE(SUM(u.sessions_count), 0)::BIGINT AS "sessions!",
            COALESCE(SUM(u.ai_requests_count), 0)::BIGINT AS "requests!",
            COALESCE(SUM(u.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(u.output_tokens), 0)::BIGINT AS "output_tokens!"
        FROM admin_usage_daily_rollups u
        WHERE u.date >= ($1::TIMESTAMPTZ AT TIME ZONE 'UTC')::DATE
          AND u.date <= ($2::TIMESTAMPTZ AT TIME ZONE 'UTC')::DATE
          AND ($3::TEXT[] IS NULL OR u.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR u.user_id = $4)
        GROUP BY 1
        ORDER BY SUM(u.ai_requests_count) DESC NULLS LAST
        LIMIT 100
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        axis.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ContainerUsageRow {
            container_id: r.container_id,
            users: r.users,
            sessions: r.sessions,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
        })
        .collect())
}
