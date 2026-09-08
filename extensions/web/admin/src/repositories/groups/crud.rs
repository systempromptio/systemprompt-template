//! Group rows: read, create, rename, delete.
//!
//! `unassigned` is a real row rather than a special case in every query, so
//! deleting it is refused here as well as by the database trigger: the repo
//! answers with a 409 a handler can return, the trigger is the backstop for
//! anything that bypasses this path.

use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::types::groups::{CreateGroupRequest, GroupRecord, GroupSummary, UpdateGroupRequest};

pub async fn list_groups(pool: &PgPool) -> Result<Vec<GroupRecord>, sqlx::Error> {
    sqlx::query_as!(
        GroupRecord,
        "SELECT id, name, description, is_system, source FROM groups ORDER BY is_system, name"
    )
    .fetch_all(pool)
    .await
}

pub async fn find_group(pool: &PgPool, group_id: &str) -> Result<Option<GroupRecord>, sqlx::Error> {
    sqlx::query_as!(
        GroupRecord,
        "SELECT id, name, description, is_system, source FROM groups WHERE id = $1",
        group_id
    )
    .fetch_optional(pool)
    .await
}

// Why: membership counts come from the `user_groups` view, so the derived
// `unassigned` members are counted the same way as everyone else and the
// listing does not have to special-case the system row.
//
// Why: the thirty-day figures are joined here rather than fetched beside the
// listing, so the JSON endpoint and the groups page cannot disagree about a
// number they both put on the same row.
pub async fn list_group_summaries(pool: &PgPool) -> Result<Vec<GroupSummary>, sqlx::Error> {
    sqlx::query_as!(
        GroupSummary,
        r#"
        SELECT
            g.id AS "id!",
            g.name AS "name!",
            g.description,
            g.is_system AS "is_system!",
            COALESCE(m.member_count, 0)::BIGINT AS "member_count!",
            COALESCE(pc.project_count, 0)::BIGINT AS "project_count!",
            COALESCE(u.active_members, 0)::BIGINT AS "active_members_30d!",
            COALESCE(u.requests, 0)::BIGINT AS "requests_30d!",
            COALESCE(u.cost_microdollars, 0)::BIGINT AS "cost_30d_microdollars!"
        FROM groups g
        LEFT JOIN (
            SELECT group_id, COUNT(DISTINCT user_id) AS member_count
            FROM user_groups GROUP BY group_id
        ) m ON m.group_id = g.id
        LEFT JOIN (
            SELECT ug.group_id, COUNT(DISTINCT pm.project_id) AS project_count
            FROM user_groups ug
            JOIN project_members pm ON pm.user_id = ug.user_id
            GROUP BY ug.group_id
        ) pc ON pc.group_id = g.id
        LEFT JOIN (
            SELECT ug.group_id,
                   COUNT(DISTINCT r.user_id) AS active_members,
                   COUNT(r.id) AS requests,
                   COALESCE(SUM(r.cost_microdollars), 0) AS cost_microdollars
            FROM user_groups ug
            JOIN ai_requests r
              ON r.user_id = ug.user_id
             AND r.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY ug.group_id
        ) u ON u.group_id = g.id
        ORDER BY g.is_system, g.name
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_group(
    pool: &PgPool,
    req: &CreateGroupRequest,
    source: &str,
) -> AdminResult<GroupRecord> {
    if find_group(pool, &req.id).await?.is_some() {
        return Err(AdminError::Conflict(format!(
            "Group {} already exists",
            req.id
        )));
    }
    sqlx::query_as!(
        GroupRecord,
        "INSERT INTO groups (id, name, description, source) VALUES ($1, $2, $3, $4)
         RETURNING id, name, description, is_system, source",
        req.id,
        req.name,
        req.description.as_deref(),
        source
    )
    .fetch_one(pool)
    .await
    .map_err(AdminError::from)
}

pub async fn update_group(
    pool: &PgPool,
    group_id: &str,
    req: &UpdateGroupRequest,
) -> AdminResult<GroupRecord> {
    sqlx::query_as!(
        GroupRecord,
        "UPDATE groups SET name = COALESCE($2, name), description = COALESCE($3, description),
         updated_at = CURRENT_TIMESTAMP WHERE id = $1
         RETURNING id, name, description, is_system, source",
        group_id,
        req.name.as_deref(),
        req.description.as_deref()
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))
}

pub async fn delete_group(pool: &PgPool, group_id: &str) -> AdminResult<()> {
    let group = find_group(pool, group_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))?;
    if group.is_system {
        return Err(AdminError::Conflict(format!(
            "Group {group_id} is a system group and cannot be deleted"
        )));
    }
    sqlx::query!("DELETE FROM groups WHERE id = $1", group_id)
        .execute(pool)
        .await?;
    Ok(())
}
