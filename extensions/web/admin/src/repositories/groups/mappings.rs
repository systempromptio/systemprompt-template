//! AD group → group mappings: what the directory's group claim resolves to.
//!
//! Rows carry `source` so the YAML loader can reconcile only what it owns:
//! a mapping added in the dashboard survives a redeploy, and one removed from
//! the YAML disappears.

use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::types::groups::GroupAdMappingRow;

pub async fn list_group_ad_mappings(
    pool: &PgPool,
    group_id: &str,
) -> Result<Vec<GroupAdMappingRow>, sqlx::Error> {
    sqlx::query_as!(
        GroupAdMappingRow,
        "SELECT ad_group, group_id, source FROM group_ad_mappings
         WHERE group_id = $1 ORDER BY ad_group",
        group_id
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_group_ad_mapping(
    pool: &PgPool,
    group_id: &str,
    ad_group: &str,
    source: &str,
) -> AdminResult<()> {
    let inserted = sqlx::query!(
        "INSERT INTO group_ad_mappings (ad_group, group_id, source) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
        ad_group,
        group_id,
        source
    )
    .execute(pool)
    .await?;
    if inserted.rows_affected() == 0 {
        return Err(AdminError::Conflict(format!(
            "{ad_group} is already mapped to {group_id}"
        )));
    }
    Ok(())
}

pub async fn delete_group_ad_mapping(
    pool: &PgPool,
    group_id: &str,
    ad_group: &str,
) -> AdminResult<()> {
    let deleted = sqlx::query!(
        "DELETE FROM group_ad_mappings WHERE group_id = $1 AND ad_group = $2",
        group_id,
        ad_group
    )
    .execute(pool)
    .await?;
    if deleted.rows_affected() == 0 {
        return Err(AdminError::NotFound(format!(
            "{ad_group} is not mapped to {group_id}"
        )));
    }
    Ok(())
}
