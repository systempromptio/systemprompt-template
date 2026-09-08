//! The groups listing's numbers, under exclusive attribution.
//!
//! The spend columns are not computed here. They come from
//! [`crate::repositories::people_usage::totals::list_scope_totals`], which
//! groups every request in the window and files what no primary group covers
//! under `unattributed` — so the rows this returns provably sum to the instance
//! total. Recomputing the same figures from a second statement would be one
//! refactor away from quietly disagreeing with the bucket that is supposed to
//! complete them.
//!
//! Everything else — membership, the projects a group's people reach, its
//! busiest model — is metadata the totals cannot carry, and is read here.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::repositories::people_usage::totals::list_scope_totals;
use crate::repositories::scope::membership::UNATTRIBUTED;

// Why: the id the remainder row carries, so a caller can split it out by name.
pub use crate::repositories::scope::membership::UNATTRIBUTED as UNATTRIBUTED_ROW;
use crate::repositories::scope::{Attribution, ScopeKind};

// Why: the listing is a page of groups, not of people, so a bound this high is
// a guard against a runaway directory rather than a paging window. The handler
// pages what it gets.
const MAX_GROUPS: i64 = 500;

/// One group on the listing: who is in it, what it reaches, what it spent.
#[derive(Debug, Clone)]
pub struct GroupUsageRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub source: String,
    pub member_count: i64,
    pub project_count: i64,
    pub active_members: i64,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub top_model: Option<String>,
    pub top_model_requests: i64,
}

// Why: Every group, plus the remainder that belongs to none of them, as one
// list.
//
// The remainder is a row rather than a second return value because that is
// what it is: under exclusive attribution these rows partition the instance,
// and the one carrying `UNATTRIBUTED_ROW` completes the partition. Returning
// it separately would invite a caller to render the groups and drop it, which
// is precisely how a listing stops adding up.
pub async fn list_groups_with_usage(
    pool: &PgPool,
    window_days: i32,
) -> Result<Vec<GroupUsageRow>, sqlx::Error> {
    let totals = list_scope_totals(pool, ScopeKind::Group, Attribution::Exclusive, window_days)
        .await?
        .into_iter()
        .map(|row| (row.scope_id.clone(), row))
        .collect::<HashMap<_, _>>();

    let meta = list_group_metadata(pool, window_days).await?;

    let mut rows: Vec<GroupUsageRow> = meta
        .into_iter()
        .map(|mut row| {
            if let Some(total) = totals.get(&row.id) {
                row.requests = total.requests;
                row.tokens = total.tokens;
                row.cost_microdollars = total.cost_microdollars;
            }
            row
        })
        .collect();

    let remainder = totals.get(UNATTRIBUTED);
    rows.push(GroupUsageRow {
        id: UNATTRIBUTED.to_owned(),
        name: "Unattributed".to_owned(),
        description: None,
        is_system: true,
        source: "system".to_owned(),
        member_count: 0,
        project_count: 0,
        active_members: 0,
        requests: remainder.map_or(0, |t| t.requests),
        tokens: remainder.map_or(0, |t| t.tokens),
        cost_microdollars: remainder.map_or(0, |t| t.cost_microdollars),
        top_model: None,
        top_model_requests: 0,
    });
    Ok(rows)
}

// Why: two attributions, one row, on purpose.
//
// `member_count` and `project_count` read `user_groups`, which is full
// membership — a person in two groups is in both, because that is what "who is
// in this group" means. `active_members` and the busiest model read the
// exclusive membership instead, so that they describe the same traffic as the
// spend columns beside them; counting a two-group person as active in both
// would put a number next to a cost that excludes them.
async fn list_group_metadata(
    pool: &PgPool,
    window_days: i32,
) -> Result<Vec<GroupUsageRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#", win AS (
               SELECT w.user_id, w.model
               FROM ai_requests w
               WHERE w.created_at >= NOW() - make_interval(days => $3)
                 AND w.actor_kind = 'user'
           ), act AS (
               SELECT m.scope_id, COUNT(DISTINCT w.user_id) AS active_members
               FROM membership m JOIN win w ON w.user_id = m.user_id
               GROUP BY 1
           ), model_rank AS (
               SELECT t.scope_id, t.model, t.requests,
                      ROW_NUMBER() OVER (
                          PARTITION BY t.scope_id ORDER BY t.requests DESC, t.model
                      ) AS rn
               FROM (
                   SELECT m.scope_id, w.model, COUNT(*) AS requests
                   FROM membership m JOIN win w ON w.user_id = m.user_id
                   WHERE w.model IS NOT NULL
                   GROUP BY 1, 2
               ) t
           )
           SELECT g.id AS "id!",
                  g.name AS "name!",
                  g.description,
                  g.is_system AS "is_system!",
                  g.source AS "source!",
                  COALESCE(mc.member_count, 0)::BIGINT AS "member_count!",
                  COALESCE(pc.project_count, 0)::BIGINT AS "project_count!",
                  COALESCE(act.active_members, 0)::BIGINT AS "active_members!",
                  model_rank.model AS "top_model?",
                  COALESCE(model_rank.requests, 0)::BIGINT AS "top_model_requests!"
           FROM groups g
           LEFT JOIN (
               SELECT ug.group_id, COUNT(DISTINCT ug.user_id) AS member_count
               FROM user_groups ug GROUP BY 1
           ) mc ON mc.group_id = g.id
           LEFT JOIN (
               SELECT ug.group_id, COUNT(DISTINCT pm.project_id) AS project_count
               FROM user_groups ug
               JOIN project_members pm ON pm.user_id = ug.user_id
               GROUP BY 1
           ) pc ON pc.group_id = g.id
           LEFT JOIN act ON act.scope_id = g.id
           LEFT JOIN model_rank ON model_rank.scope_id = g.id AND model_rank.rn = 1
           ORDER BY g.is_system, g.name
           LIMIT $4"#,
        ScopeKind::Group.as_str(),
        Attribution::Exclusive.is_exclusive(),
        window_days,
        MAX_GROUPS
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| GroupUsageRow {
            id: row.id,
            name: row.name,
            description: row.description,
            is_system: row.is_system,
            source: row.source,
            member_count: row.member_count,
            project_count: row.project_count,
            active_members: row.active_members,
            requests: 0,
            tokens: 0,
            cost_microdollars: 0,
            top_model: row.top_model,
            top_model_requests: row.top_model_requests,
        })
        .collect())
}
