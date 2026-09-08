//! Project rows: read, create, rename, delete.
//!
//! No system row and no delete refusal: a project is work attribution, and
//! deleting one cascades its membership away, which is what an admin removing
//! a finished project means.

use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::types::projects::{
    CreateProjectRequest, ProjectRow, ProjectSummary, UpdateProjectRequest,
};

pub async fn list_projects(pool: &PgPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectRow,
        "SELECT id, name, description, source FROM projects ORDER BY name"
    )
    .fetch_all(pool)
    .await
}

pub async fn find_project(
    pool: &PgPool,
    project_id: &str,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectRow,
        "SELECT id, name, description, source FROM projects WHERE id = $1",
        project_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_project_summaries(pool: &PgPool) -> Result<Vec<ProjectSummary>, sqlx::Error> {
    sqlx::query_as!(
        ProjectSummary,
        r#"
        SELECT
            p.id AS "id!",
            p.name AS "name!",
            p.description,
            COALESCE(m.member_count, 0)::BIGINT AS "member_count!",
            COALESCE(u.active_members, 0)::BIGINT AS "active_members_30d!",
            COALESCE(u.requests, 0)::BIGINT AS "requests_30d!",
            COALESCE(u.cost_microdollars, 0)::BIGINT AS "cost_30d_microdollars!"
        FROM projects p
        LEFT JOIN (
            SELECT project_id, COUNT(DISTINCT user_id) AS member_count
            FROM project_members GROUP BY project_id
        ) m ON m.project_id = p.id
        LEFT JOIN (
            SELECT pm.project_id,
                   COUNT(DISTINCT r.user_id) AS active_members,
                   COUNT(r.id) AS requests,
                   COALESCE(SUM(r.cost_microdollars), 0) AS cost_microdollars
            FROM (SELECT DISTINCT project_id, user_id FROM project_members) pm
            JOIN ai_requests r
              ON r.user_id = pm.user_id
             AND r.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY pm.project_id
        ) u ON u.project_id = p.id
        ORDER BY p.name
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_project(
    pool: &PgPool,
    req: &CreateProjectRequest,
    source: &str,
) -> AdminResult<ProjectRow> {
    if find_project(pool, &req.id).await?.is_some() {
        return Err(AdminError::Conflict(format!(
            "Project {} already exists",
            req.id
        )));
    }
    sqlx::query_as!(
        ProjectRow,
        "INSERT INTO projects (id, name, description, source) VALUES ($1, $2, $3, $4)
         RETURNING id, name, description, source",
        req.id,
        req.name,
        req.description.as_deref(),
        source
    )
    .fetch_one(pool)
    .await
    .map_err(AdminError::from)
}

pub async fn update_project(
    pool: &PgPool,
    project_id: &str,
    req: &UpdateProjectRequest,
) -> AdminResult<ProjectRow> {
    sqlx::query_as!(
        ProjectRow,
        "UPDATE projects SET name = COALESCE($2, name),
         description = COALESCE($3, description), updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 RETURNING id, name, description, source",
        project_id,
        req.name.as_deref(),
        req.description.as_deref()
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AdminError::NotFound(format!("Project {project_id} not found")))
}

pub async fn delete_project(pool: &PgPool, project_id: &str) -> AdminResult<()> {
    let deleted = sqlx::query!("DELETE FROM projects WHERE id = $1", project_id)
        .execute(pool)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(AdminError::NotFound(format!(
            "Project {project_id} not found"
        )));
    }
    Ok(())
}
