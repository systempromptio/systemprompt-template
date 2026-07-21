//! Distinct-value lookups for the identity-filter-ribbon partial.
//!
//! Aggregated over a `TimeRange` so the dropdowns only show identities/policies
//! that actually appear in the user's current view. `tenant_id` lives on
//! `ai_requests`, not `governance_decisions`, so the tenants list is sourced
//! from `ai_requests` over the same window.

use serde::Serialize;
use sqlx::PgPool;

use crate::util::time_range::TimeRange;

#[derive(Debug, Clone, Serialize)]
pub struct FilterOption {
    pub id: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FilterOptions {
    pub users: Vec<FilterOption>,
    pub agents: Vec<FilterOption>,
    pub agent_scopes: Vec<FilterOption>,
    pub policies: Vec<FilterOption>,
    pub decisions: Vec<FilterOption>,
}

const PER_FACET_LIMIT: i64 = 100;

pub async fn fetch_filter_options(
    pool: &PgPool,
    range: TimeRange,
) -> Result<FilterOptions, sqlx::Error> {
    let users = fetch_users(pool, range).await?;
    let agents = fetch_agents(pool, range).await?;
    let agent_scopes = fetch_agent_scopes(pool, range).await?;
    let policies = fetch_policies(pool, range).await?;
    let decisions = fetch_decisions(pool, range).await?;

    Ok(FilterOptions {
        users,
        agents,
        agent_scopes,
        policies,
        decisions,
    })
}

async fn fetch_users(pool: &PgPool, range: TimeRange) -> Result<Vec<FilterOption>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT g.user_id as "id!",
                  COALESCE(u.display_name, u.full_name, u.name, u.email, g.user_id) as "label!",
                  COUNT(*)::bigint as "count!"
           FROM governance_decisions g
           LEFT JOIN users u ON u.id = g.user_id
           WHERE g.created_at >= $1 AND g.created_at < $2
           GROUP BY g.user_id, u.display_name, u.full_name, u.name, u.email
           ORDER BY COUNT(*) DESC
           LIMIT $3"#,
        range.from,
        range.to,
        PER_FACET_LIMIT,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| FilterOption {
        id: r.id,
        label: r.label,
        count: r.count,
    })
    .collect())
}

async fn fetch_agents(pool: &PgPool, range: TimeRange) -> Result<Vec<FilterOption>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT agent_id as "id!",
                  agent_id as "label!",
                  COUNT(*)::bigint as "count!"
           FROM governance_decisions
           WHERE agent_id IS NOT NULL
             AND created_at >= $1 AND created_at < $2
           GROUP BY agent_id
           ORDER BY COUNT(*) DESC
           LIMIT $3"#,
        range.from,
        range.to,
        PER_FACET_LIMIT,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| FilterOption {
        id: r.id,
        label: r.label,
        count: r.count,
    })
    .collect())
}

async fn fetch_agent_scopes(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<FilterOption>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT agent_scope as "id!",
                  agent_scope as "label!",
                  COUNT(*)::bigint as "count!"
           FROM governance_decisions
           WHERE agent_scope IS NOT NULL
             AND created_at >= $1 AND created_at < $2
           GROUP BY agent_scope
           ORDER BY COUNT(*) DESC
           LIMIT $3"#,
        range.from,
        range.to,
        PER_FACET_LIMIT,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| FilterOption {
        id: r.id,
        label: r.label,
        count: r.count,
    })
    .collect())
}

async fn fetch_policies(pool: &PgPool, range: TimeRange) -> Result<Vec<FilterOption>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT policy as "id!",
                  policy as "label!",
                  COUNT(*)::bigint as "count!"
           FROM governance_decisions
           WHERE created_at >= $1 AND created_at < $2
           GROUP BY policy
           ORDER BY COUNT(*) DESC
           LIMIT $3"#,
        range.from,
        range.to,
        PER_FACET_LIMIT,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| FilterOption {
        id: r.id,
        label: r.label,
        count: r.count,
    })
    .collect())
}

async fn fetch_decisions(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<FilterOption>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT decision as "id!",
                  decision as "label!",
                  COUNT(*)::bigint as "count!"
           FROM governance_decisions
           WHERE created_at >= $1 AND created_at < $2
           GROUP BY decision
           ORDER BY COUNT(*) DESC"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| FilterOption {
        id: r.id,
        label: r.label,
        count: r.count,
    })
    .collect())
}
