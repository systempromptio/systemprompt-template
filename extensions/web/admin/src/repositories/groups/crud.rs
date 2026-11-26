//! Group rows: read, create, rename, delete.
//!
//! `unassigned` is a real row rather than a special case in every query, so
//! deleting it is refused here as well as by the database trigger: the repo
//! answers with a 409 a handler can return, the trigger is the backstop for
//! anything that bypasses this path.

use sqlx::PgPool;
use systemprompt_web_shared::GroupId;

use crate::error::{AdminError, AdminResult};
#[cfg(feature = "gateway")]
use crate::types::Usage30d;
use crate::types::groups::{CreateGroupRequest, GroupRecord, GroupSummary, UpdateGroupRequest};

pub async fn list_groups(pool: &PgPool) -> Result<Vec<GroupRecord>, sqlx::Error> {
    sqlx::query_as!(
        GroupRecord,
        r#"SELECT id AS "id: GroupId", name, description, is_system, source FROM groups ORDER BY is_system, name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn find_group(
    pool: &PgPool,
    group_id: &GroupId,
) -> Result<Option<GroupRecord>, sqlx::Error> {
    sqlx::query_as!(
        GroupRecord,
        r#"SELECT id AS "id: GroupId", name, description, is_system, source FROM groups WHERE id = $1"#,
        group_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

// Why: membership counts come from the `user_groups` view, so the derived
// `unassigned` members are counted the same way as everyone else and the
// listing does not have to special-case the system row. The thirty-day
// gateway figures are a keyed read beside it in the builds that link the
// gateway member, so the JSON endpoint and the groups page read one number.
pub async fn list_group_summaries(pool: &PgPool) -> Result<Vec<GroupSummary>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            g.id AS "id!: GroupId",
            g.name AS "name!",
            g.description,
            g.is_system AS "is_system!",
            COALESCE(m.member_count, 0)::BIGINT AS "member_count!",
            COALESCE(pc.project_count, 0)::BIGINT AS "project_count!"
        FROM groups g
        LEFT JOIN (
            SELECT group_id, COUNT(DISTINCT user_id) AS member_count
            FROM user_groups GROUP BY group_id
        ) m ON m.group_id = g.id
        LEFT JOIN (
            SELECT ug.group_id, COUNT(DISTINCT pm.project_id) AS project_count
            FROM user_groups ug
            JOIN user_projects pm ON pm.user_id = ug.user_id
            GROUP BY ug.group_id
        ) pc ON pc.group_id = g.id
        ORDER BY g.is_system, g.name
        "#
    )
    .fetch_all(pool)
    .await?;
    #[cfg(feature = "gateway")]
    let usage = list_group_usage_30d(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| GroupSummary {
            #[cfg(feature = "gateway")]
            usage: usage.get(r.id.as_str()).copied().unwrap_or_default(),
            id: r.id,
            name: r.name,
            description: r.description,
            is_system: r.is_system,
            member_count: r.member_count,
            project_count: r.project_count,
        })
        .collect())
}

#[cfg(feature = "gateway")]
async fn list_group_usage_30d(
    pool: &PgPool,
) -> Result<std::collections::HashMap<String, Usage30d>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT ug.group_id AS "group_id!",
               COUNT(DISTINCT r.user_id)::BIGINT AS "active_members!",
               COUNT(r.id)::BIGINT AS "requests!",
               COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!"
        FROM user_groups ug
        JOIN ai_requests r
          ON r.user_id = ug.user_id
         AND r.created_at >= NOW() - INTERVAL '30 days'
        GROUP BY ug.group_id
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.group_id,
                Usage30d {
                    active_members_30d: r.active_members,
                    requests_30d: r.requests,
                    cost_30d_microdollars: r.cost_microdollars,
                },
            )
        })
        .collect())
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
        r#"INSERT INTO groups (id, name, description, source) VALUES ($1, $2, $3, $4)
         RETURNING id AS "id: GroupId", name, description, is_system, source"#,
        req.id.as_str(),
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
    group_id: &GroupId,
    req: &UpdateGroupRequest,
) -> AdminResult<GroupRecord> {
    sqlx::query_as!(
        GroupRecord,
        r#"UPDATE groups SET name = COALESCE($2, name), description = COALESCE($3, description),
         updated_at = CURRENT_TIMESTAMP WHERE id = $1
         RETURNING id AS "id: GroupId", name, description, is_system, source"#,
        group_id.as_str(),
        req.name.as_deref(),
        req.description.as_deref()
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))
}

pub async fn delete_group(pool: &PgPool, group_id: &GroupId) -> AdminResult<()> {
    let group = find_group(pool, group_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("Group {group_id} not found")))?;
    if group.is_system {
        return Err(AdminError::Conflict(format!(
            "Group {group_id} is a system group and cannot be deleted"
        )));
    }
    sqlx::query!("DELETE FROM groups WHERE id = $1", group_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}
