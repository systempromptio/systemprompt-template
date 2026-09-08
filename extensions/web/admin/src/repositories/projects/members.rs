//! Project membership, with the same two-writer rule groups have: the
//! directory replaces its own rows at every sign-in, manual rows survive.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::types::projects::ProjectMemberRow;

pub async fn list_project_members(
    pool: &PgPool,
    project_id: &str,
) -> Result<Vec<ProjectMemberRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectMemberRow,
        r#"
        SELECT
            pm.user_id AS "user_id!: UserId",
            u.display_name AS "display_name?",
            u.email AS "email?",
            ARRAY_AGG(DISTINCT pm.source) AS "sources!",
            COALESCE(
                ARRAY_AGG(DISTINCT pm.source_ad_group)
                    FILTER (WHERE pm.source_ad_group IS NOT NULL),
                ARRAY[]::TEXT[]
            ) AS "source_ad_groups!"
        FROM project_members pm
        JOIN users u ON u.id = pm.user_id
        WHERE pm.project_id = $1
        GROUP BY pm.user_id, u.display_name, u.email
        ORDER BY u.display_name NULLS LAST, pm.user_id
        "#,
        project_id
    )
    .fetch_all(pool)
    .await
}

pub async fn list_project_ids_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT DISTINCT project_id FROM project_members WHERE user_id = $1 ORDER BY project_id",
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_project_member(
    pool: &PgPool,
    project_id: &str,
    user_id: &UserId,
    granted_by: &UserId,
) -> AdminResult<()> {
    let inserted = sqlx::query!(
        "INSERT INTO project_members (project_id, user_id, source, granted_by)
         VALUES ($1, $2, 'manual', $3) ON CONFLICT DO NOTHING",
        project_id,
        user_id.as_str(),
        granted_by.as_str()
    )
    .execute(pool)
    .await?;
    if inserted.rows_affected() == 0 {
        return Err(AdminError::Conflict(format!(
            "User {user_id} is already a manual member of {project_id}"
        )));
    }
    Ok(())
}

pub async fn delete_project_member(
    pool: &PgPool,
    project_id: &str,
    user_id: &UserId,
) -> AdminResult<()> {
    let sources = sqlx::query_scalar!(
        r#"SELECT source AS "source!" FROM project_members
           WHERE project_id = $1 AND user_id = $2"#,
        project_id,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await?;

    if sources.is_empty() {
        return Err(AdminError::NotFound(format!(
            "User {user_id} is not a member of {project_id}"
        )));
    }
    if !sources.iter().any(|s| s == "manual") {
        return Err(AdminError::Conflict(format!(
            "User {user_id} is in {project_id} through the directory; remove them from the AD group"
        )));
    }
    sqlx::query!(
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2 AND source = 'manual'",
        project_id,
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: lint-ok: unused-pub — called by the downstream ADFS sign-in integration.
pub async fn replace_directory_project_memberships(
    pool: &PgPool,
    user_id: &UserId,
    ad_groups: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM project_members WHERE user_id = $1 AND source = 'adfs'",
        user_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO project_members (project_id, user_id, source, source_ad_group)
         SELECT m.project_id, $1, 'adfs', m.ad_group FROM project_ad_mappings m
         WHERE m.ad_group = ANY($2) ON CONFLICT DO NOTHING",
        user_id.as_str(),
        ad_groups
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    crate::repositories::scope::defaults::recompute_scope_defaults_for_user(pool, user_id).await?;
    Ok(())
}
