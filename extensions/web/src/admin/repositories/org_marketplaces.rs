use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::marketplaces::{
    CreateOrgMarketplaceRequest, OrgMarketplace, UpdateOrgMarketplaceRequest,
};

pub async fn list_org_marketplaces(pool: &Arc<PgPool>) -> Result<Vec<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as!(
        OrgMarketplace,
        "SELECT id, name, description, enabled, created_at, updated_at
         FROM org_marketplaces
         ORDER BY name",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn find_org_marketplace(
    pool: &Arc<PgPool>,
    id: &str,
) -> Result<Option<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as!(
        OrgMarketplace,
        "SELECT id, name, description, enabled, created_at, updated_at
         FROM org_marketplaces
         WHERE id = $1",
        id,
    )
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn create_org_marketplace(
    pool: &Arc<PgPool>,
    req: &CreateOrgMarketplaceRequest,
) -> Result<OrgMarketplace, sqlx::Error> {
    let marketplace = sqlx::query_as!(
        OrgMarketplace,
        "INSERT INTO org_marketplaces (id, name, description, enabled)
         VALUES ($1, $2, $3, true)
         RETURNING id, name, description, enabled, created_at, updated_at",
        req.id,
        req.name,
        req.description,
    )
    .fetch_one(pool.as_ref())
    .await?;

    if !req.plugin_ids.is_empty() {
        set_marketplace_plugins(pool, &req.id, &req.plugin_ids).await?;
    }

    Ok(marketplace)
}

pub async fn update_org_marketplace(
    pool: &Arc<PgPool>,
    id: &str,
    req: &UpdateOrgMarketplaceRequest,
) -> Result<Option<OrgMarketplace>, sqlx::Error> {
    let Some(existing) = find_org_marketplace(pool, id).await? else {
        return Ok(None);
    };

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let description = req.description.as_deref().unwrap_or(&existing.description);

    let updated = sqlx::query_as!(
        OrgMarketplace,
        "UPDATE org_marketplaces
         SET name = $2, description = $3, updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, description, enabled, created_at, updated_at",
        id,
        name,
        description,
    )
    .fetch_one(pool.as_ref())
    .await?;

    if let Some(ref plugin_ids) = req.plugin_ids {
        set_marketplace_plugins(pool, id, plugin_ids).await?;
    }

    Ok(Some(updated))
}

pub async fn delete_org_marketplace(pool: &Arc<PgPool>, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM org_marketplaces WHERE id = $1", id)
        .execute(pool.as_ref())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_marketplace_plugin_ids(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT plugin_id FROM org_marketplace_plugins
         WHERE marketplace_id = $1
         ORDER BY position, created_at",
        marketplace_id,
    )
    .fetch_all(pool.as_ref())
    .await?;
    Ok(rows.into_iter().map(|r| r.plugin_id).collect())
}

pub async fn set_marketplace_plugins(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    plugin_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM org_marketplace_plugins WHERE marketplace_id = $1",
        marketplace_id,
    )
    .execute(&mut *tx)
    .await?;

    for (i, plugin_id) in plugin_ids.iter().enumerate() {
        sqlx::query!(
            "INSERT INTO org_marketplace_plugins (marketplace_id, plugin_id, position)
             VALUES ($1, $2, $3)",
            marketplace_id,
            plugin_id,
            i32::try_from(i).unwrap_or(0),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn resolve_authorized_org_plugin_ids(
    pool: &Arc<PgPool>,
) -> Result<HashSet<String>, sqlx::Error> {
    let marketplaces = list_org_marketplaces(pool).await?;
    let mut org_plugin_ids: HashSet<String> = HashSet::new();
    for mkt in &marketplaces {
        if let Ok(plugin_ids) = list_marketplace_plugin_ids(pool, &mkt.id).await {
            for pid in plugin_ids {
                org_plugin_ids.insert(pid);
            }
        }
    }
    org_plugin_ids.insert("systemprompt".to_string());
    Ok(org_plugin_ids)
}

pub async fn resolve_authorized_marketplace_groups(
    pool: &Arc<PgPool>,
) -> Result<Vec<(OrgMarketplace, Vec<String>)>, sqlx::Error> {
    let marketplaces = list_org_marketplaces(pool).await?;
    let mut result: Vec<(OrgMarketplace, Vec<String>)> = Vec::new();
    for mkt in marketplaces {
        let plugin_ids = list_marketplace_plugin_ids(pool, &mkt.id).await?;
        result.push((mkt, plugin_ids));
    }
    Ok(result)
}

pub async fn list_marketplaces_for_plugins(
    pool: &Arc<PgPool>,
) -> Result<HashMap<String, Vec<(String, String)>>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT mp.plugin_id, m.id, m.name
         FROM org_marketplace_plugins mp
         JOIN org_marketplaces m ON m.id = mp.marketplace_id
         ORDER BY m.name",
    )
    .fetch_all(pool.as_ref())
    .await?;

    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for row in rows {
        map.entry(row.plugin_id)
            .or_default()
            .push((row.id, row.name));
    }
    Ok(map)
}

/// Lists all org marketplaces that have a GitHub repo URL configured.
pub async fn list_github_marketplaces(
    pool: &Arc<PgPool>,
) -> Result<Vec<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as::<_, OrgMarketplace>(
        "SELECT id, name, description, github_repo_url, enabled, created_at, updated_at
         FROM org_marketplaces
         WHERE github_repo_url IS NOT NULL AND github_repo_url != ''
         ORDER BY name",
    )
    .fetch_all(pool.as_ref())
    .await
}

/// Insert a sync log entry for a marketplace operation.
#[allow(clippy::too_many_arguments)]
pub async fn insert_sync_log(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    operation: &str,
    status: &str,
    commit_hash: Option<&str>,
    plugins_synced: i64,
    errors: i64,
    error_message: Option<&str>,
    triggered_by: &str,
    duration_ms: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO org_marketplace_sync_logs (marketplace_id, operation, status, commit_hash, plugins_synced, errors, error_message, triggered_by, duration_ms)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(marketplace_id)
    .bind(operation)
    .bind(status)
    .bind(commit_hash)
    .bind(plugins_synced)
    .bind(errors)
    .bind(error_message)
    .bind(triggered_by)
    .bind(duration_ms)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}
