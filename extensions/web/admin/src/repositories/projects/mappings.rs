//! AD group → project mappings, reconciled by source the way group mappings
//! are: the YAML loader owns `yaml` rows, the dashboard owns the rest.

use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::types::projects::ProjectAdMappingRow;

pub async fn list_project_ad_mappings(
    pool: &PgPool,
    project_id: &str,
) -> Result<Vec<ProjectAdMappingRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectAdMappingRow,
        "SELECT ad_group, project_id, source FROM project_ad_mappings
         WHERE project_id = $1 ORDER BY ad_group",
        project_id
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_project_ad_mapping(
    pool: &PgPool,
    project_id: &str,
    ad_group: &str,
    source: &str,
) -> AdminResult<()> {
    let inserted = sqlx::query!(
        "INSERT INTO project_ad_mappings (ad_group, project_id, source) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
        ad_group,
        project_id,
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
    project_id: &str,
    ad_group: &str,
) -> AdminResult<()> {
    let deleted = sqlx::query!(
        "DELETE FROM project_ad_mappings WHERE project_id = $1 AND ad_group = $2",
        project_id,
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
