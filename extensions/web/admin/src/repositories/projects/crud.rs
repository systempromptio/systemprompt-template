//! Project rows: read, create, rename, delete.
//!
//! No system row and no delete refusal: a project is work attribution, and
//! deleting one cascades its membership away, which is what an admin removing
//! a finished project means.

use sqlx::PgPool;
use systemprompt_web_shared::ProjectId;

use crate::error::{AdminError, AdminResult};
use crate::types::projects::{
    CreateProjectRequest, ProjectRow, ProjectSummary, UpdateProjectRequest,
};

pub async fn list_projects(pool: &PgPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectRow,
        r#"SELECT id AS "id: ProjectId", name, description, source FROM projects ORDER BY name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn find_project(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectRow,
        r#"SELECT id AS "id: ProjectId", name, description, source FROM projects WHERE id = $1"#,
        project_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_project_summaries(pool: &PgPool) -> Result<Vec<ProjectSummary>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            p.id AS "id!: ProjectId",
            p.name AS "name!",
            p.description,
            COALESCE(m.member_count, 0)::BIGINT AS "member_count!"
        FROM projects p
        LEFT JOIN (
            SELECT project_id, COUNT(*) AS member_count
            FROM user_projects GROUP BY project_id
        ) m ON m.project_id = p.id
        ORDER BY p.name
        "#
    )
    .fetch_all(pool)
    .await?;
    #[cfg(feature = "gateway")]
    let usage = list_project_usage_30d(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| ProjectSummary {
            #[cfg(feature = "gateway")]
            usage: usage.get(r.id.as_str()).copied().unwrap_or_default(),
            id: r.id,
            name: r.name,
            description: r.description,
            member_count: r.member_count,
        })
        .collect())
}

// Why: the thirty-day gateway figures, keyed by project, in the builds that
// link the gateway member.
#[cfg(feature = "gateway")]
async fn list_project_usage_30d(
    pool: &PgPool,
) -> Result<std::collections::HashMap<String, crate::types::Usage30d>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT pm.project_id AS "project_id!",
               COUNT(DISTINCT r.user_id)::BIGINT AS "active_members!",
               COUNT(r.id)::BIGINT AS "requests!",
               COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!"
        FROM user_projects pm
        JOIN ai_requests r
          ON r.user_id = pm.user_id
         AND r.created_at >= NOW() - INTERVAL '30 days'
        GROUP BY pm.project_id
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.project_id,
                crate::types::Usage30d {
                    active_members_30d: r.active_members,
                    requests_30d: r.requests,
                    cost_30d_microdollars: r.cost_microdollars,
                },
            )
        })
        .collect())
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
        r#"INSERT INTO projects (id, name, description, source) VALUES ($1, $2, $3, $4)
         RETURNING id AS "id: ProjectId", name, description, source"#,
        req.id.as_str(),
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
    project_id: &ProjectId,
    req: &UpdateProjectRequest,
) -> AdminResult<ProjectRow> {
    sqlx::query_as!(
        ProjectRow,
        r#"UPDATE projects SET name = COALESCE($2, name),
         description = COALESCE($3, description), updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 RETURNING id AS "id: ProjectId", name, description, source"#,
        project_id.as_str(),
        req.name.as_deref(),
        req.description.as_deref()
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AdminError::NotFound(format!("Project {project_id} not found")))
}

pub async fn delete_project(pool: &PgPool, project_id: &ProjectId) -> AdminResult<()> {
    let deleted = sqlx::query!("DELETE FROM projects WHERE id = $1", project_id.as_str())
        .execute(pool)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(AdminError::NotFound(format!(
            "Project {project_id} not found"
        )));
    }
    Ok(())
}
