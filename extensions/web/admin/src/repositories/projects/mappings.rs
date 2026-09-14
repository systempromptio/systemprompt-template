//! AD group → project mappings, reconciled by source the way group mappings
//! are: the YAML loader owns `yaml` rows, the dashboard owns the rest.

use sqlx::PgPool;
use systemprompt_web_shared::ProjectId;

use crate::error::{AdminError, AdminResult};
use crate::types::projects::ProjectAdMappingRow;

pub async fn list_project_ad_mappings(
    pool: &PgPool,
    project_id: &ProjectId,
) -> Result<Vec<ProjectAdMappingRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectAdMappingRow,
        r#"SELECT ad_group, project_id AS "project_id: ProjectId", source FROM project_ad_mappings
         WHERE project_id = $1 ORDER BY ad_group"#,
        project_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_project_ad_mapping(
    pool: &PgPool,
    project_id: &ProjectId,
    ad_group: &str,
    source: &str,
) -> AdminResult<()> {
    let inserted = sqlx::query!(
        "INSERT INTO project_ad_mappings (ad_group, project_id, source) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
        ad_group,
        project_id.as_str(),
        source
    )
    .execute(pool)
    .await?;
    if inserted.rows_affected() == 0 {
        return Err(AdminError::Conflict(format!(
            "{ad_group} is already mapped to {project_id}"
        )));
    }
    Ok(())
}

pub async fn delete_project_ad_mapping(
    pool: &PgPool,
    project_id: &ProjectId,
    ad_group: &str,
) -> AdminResult<()> {
    let deleted = sqlx::query!(
        "DELETE FROM project_ad_mappings WHERE project_id = $1 AND ad_group = $2",
        project_id.as_str(),
        ad_group
    )
    .execute(pool)
    .await?;
    if deleted.rows_affected() == 0 {
        return Err(AdminError::NotFound(format!(
            "{ad_group} is not mapped to {project_id}"
        )));
    }
    Ok(())
}
