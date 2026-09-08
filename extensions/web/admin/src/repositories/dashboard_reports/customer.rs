//! A project's month, as its people are allowed to see it.
//!
//! Nothing here selects `cost_microdollars`. What we paid a provider is an
//! internal figure, and a report that carries it is one screenshot away from
//! becoming a pricing discussion. Keeping the column out of the query — rather
//! than out of the template — means the guarantee survives someone adding a
//! field to the view-model later.
//!
//! The scope is a [`SubjectScope`]: every user, or the id list the caller may
//! see. A user in no project folds into `unassigned` rather than dropping out
//! of the totals.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

use crate::repositories::scope::SubjectScope;

/// The header figures: who was active and what they used.
#[derive(Debug, Clone, Copy)]
pub struct CustomerMonthSummary {
    // Why: Users that actually made a request in the month.
    pub active_users: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    // Why: reasoning tokens are billed inside `output_tokens`, so this is a
    // share of that column, never an addition to it.
    pub reasoning_tokens: i64,
    // Why: `tokens_used` as written by `CanonicalUsage::billable_total()` --
    // the one definition of a request's billable total. Summing the component
    // columns here would double-count the cached slice.
    pub total_tokens: i64,
    pub error_count: i64,
}

pub async fn get_customer_month_summary(
    pool: &PgPool,
    scope: &SubjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<CustomerMonthSummary, MarketplaceError> {
    let r = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT AS "requests!",
            COUNT(DISTINCT r.user_id)::BIGINT AS "active_users!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(r.cache_read_tokens), 0)::BIGINT AS "cache_read_tokens!",
            COALESCE(SUM(r.reasoning_tokens), 0)::BIGINT AS "reasoning_tokens!",
            COALESCE(SUM(r.tokens_used), 0)::BIGINT AS "total_tokens!",
            COUNT(*) FILTER (WHERE r.status NOT IN ('success', 'completed'))::BIGINT
                AS "error_count!"
        FROM ai_requests r
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_one(pool)
    .await?;

    Ok(CustomerMonthSummary {
        active_users: r.active_users,
        requests: r.requests,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        cache_read_tokens: r.cache_read_tokens,
        reasoning_tokens: r.reasoning_tokens,
        total_tokens: r.total_tokens,
        error_count: r.error_count,
    })
}

/// One user's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerUserUsage {
    pub email: String,
    pub display_name: String,
    pub project: Option<String>,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
    pub distinct_models: i64,
}

// Why: Users with no activity are omitted: their row would be a line of
// zeroes, and on a busy month that is most of the table.
pub async fn list_customer_month_users(
    pool: &PgPool,
    scope: &SubjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerUserUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            u.email AS "email!",
            COALESCE(NULLIF(u.display_name, ''), u.name) AS "display_name!",
            NULLIF(ARRAY_TO_STRING(ARRAY(
                SELECT DISTINCT pm.project_id FROM project_members pm
                WHERE pm.user_id = u.id), ', '), '') AS "project?",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(r.reasoning_tokens), 0)::BIGINT AS "reasoning_tokens!",
            COALESCE(SUM(r.tokens_used), 0)::BIGINT AS "total_tokens!",
            COUNT(DISTINCT r.model)::BIGINT AS "distinct_models!"
        FROM ai_requests r
        JOIN users u ON u.id = r.user_id
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
        GROUP BY u.id, u.email, u.display_name, u.name
        ORDER BY COALESCE(SUM(r.tokens_used), 0) DESC, COUNT(*) DESC
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerUserUsage {
            email: r.email,
            display_name: r.display_name,
            project: r.project,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            reasoning_tokens: r.reasoning_tokens,
            total_tokens: r.total_tokens,
            distinct_models: r.distinct_models,
        })
        .collect())
}

/// One project's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerProjectUsage {
    pub project: String,
    pub members: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
}

pub async fn list_customer_month_projects(
    pool: &PgPool,
    scope: &SubjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerProjectUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(pm.project_id, 'unassigned') AS "project!",
            COUNT(DISTINCT r.user_id)::BIGINT AS "members!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(r.reasoning_tokens), 0)::BIGINT AS "reasoning_tokens!",
            COALESCE(SUM(r.tokens_used), 0)::BIGINT AS "total_tokens!"
        FROM ai_requests r
        LEFT JOIN LATERAL (
            SELECT DISTINCT project_id FROM project_members
            WHERE user_id = r.user_id
        ) pm ON true
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
        GROUP BY 1
        ORDER BY COALESCE(SUM(r.tokens_used), 0) DESC
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerProjectUsage {
            project: r.project,
            members: r.members,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            reasoning_tokens: r.reasoning_tokens,
            total_tokens: r.total_tokens,
        })
        .collect())
}

/// One model's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerModelUsage {
    pub provider: String,
    pub model: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
}

pub async fn list_customer_month_models(
    pool: &PgPool,
    scope: &SubjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerModelUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(r.provider, 'unrouted') AS "provider!",
            COALESCE(r.model, 'unrouted') AS "model!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(r.cache_read_tokens), 0)::BIGINT AS "cache_read_tokens!",
            COALESCE(SUM(r.reasoning_tokens), 0)::BIGINT AS "reasoning_tokens!",
            COALESCE(SUM(r.tokens_used), 0)::BIGINT AS "total_tokens!"
        FROM ai_requests r
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
        GROUP BY 1, 2
        ORDER BY COALESCE(SUM(r.tokens_used), 0) DESC
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerModelUsage {
            provider: r.provider,
            model: r.model,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_read_tokens: r.cache_read_tokens,
            reasoning_tokens: r.reasoning_tokens,
            total_tokens: r.total_tokens,
        })
        .collect())
}
