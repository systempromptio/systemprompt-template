use std::collections::{HashMap, HashSet};

use sqlx::PgPool;

use super::super::types::marketplaces::{
    CreateOrgMarketplaceRequest, OrgMarketplace, UpdateOrgMarketplaceRequest,
};

pub async fn list_org_marketplaces(pool: &PgPool) -> Result<Vec<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as!(
        OrgMarketplace,
        "SELECT id, name, description, github_repo_url, enabled, created_at, updated_at
         FROM org_marketplaces
         ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn find_org_marketplace(
    pool: &PgPool,
    id: &str,
) -> Result<Option<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as!(
        OrgMarketplace,
        "SELECT id, name, description, github_repo_url, enabled, created_at, updated_at
         FROM org_marketplaces
         WHERE id = $1",
        id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_org_marketplace(
    pool: &PgPool,
    req: &CreateOrgMarketplaceRequest,
) -> Result<OrgMarketplace, sqlx::Error> {
    let marketplace = sqlx::query_as!(
        OrgMarketplace,
        "INSERT INTO org_marketplaces (id, name, description, enabled)
         VALUES ($1, $2, $3, true)
         RETURNING id, name, description, github_repo_url, enabled, created_at, updated_at",
        req.id,
        req.name,
        req.description,
    )
    .fetch_one(pool)
    .await?;

    if !req.plugin_ids.is_empty() {
        set_marketplace_plugins(pool, &req.id, &req.plugin_ids).await?;
    }

    Ok(marketplace)
}

pub async fn update_org_marketplace(
    pool: &PgPool,
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
         RETURNING id, name, description, github_repo_url, enabled, created_at, updated_at",
        id,
        name,
        description,
    )
    .fetch_one(pool)
    .await?;

    if let Some(ref plugin_ids) = req.plugin_ids {
        set_marketplace_plugins(pool, id, plugin_ids).await?;
    }

    Ok(Some(updated))
}

pub async fn delete_org_marketplace(pool: &PgPool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM org_marketplaces WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_marketplace_plugin_ids(
    pool: &PgPool,
    marketplace_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT plugin_id FROM org_marketplace_plugins
         WHERE marketplace_id = $1
         ORDER BY position, created_at",
        marketplace_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.plugin_id).collect())
}

pub async fn set_marketplace_plugins(
    pool: &PgPool,
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
    pool: &PgPool,
) -> Result<HashSet<String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT DISTINCT mp.plugin_id
         FROM org_marketplace_plugins mp
         JOIN org_marketplaces m ON m.id = mp.marketplace_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.plugin_id).collect())
}

pub async fn resolve_authorized_marketplace_groups(
    pool: &PgPool,
) -> Result<Vec<(OrgMarketplace, Vec<String>)>, sqlx::Error> {
    let (marketplaces, assoc_rows) = tokio::join!(
        list_org_marketplaces(pool),
        sqlx::query!(
            "SELECT marketplace_id, plugin_id
             FROM org_marketplace_plugins
             ORDER BY marketplace_id, position, created_at",
        )
        .fetch_all(pool),
    );
    let marketplaces = marketplaces?;
    let assoc_rows = assoc_rows?;

    let mut plugin_map: HashMap<String, Vec<String>> = HashMap::new();
    for row in assoc_rows {
        plugin_map
            .entry(row.marketplace_id)
            .or_default()
            .push(row.plugin_id);
    }

    let result = marketplaces
        .into_iter()
        .map(|mkt| {
            let plugins = plugin_map.remove(&mkt.id).unwrap_or_default();
            (mkt, plugins)
        })
        .collect();
    Ok(result)
}

pub async fn list_marketplaces_for_plugins(
    pool: &PgPool,
) -> Result<HashMap<String, Vec<(String, String)>>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT mp.plugin_id, m.id, m.name
         FROM org_marketplace_plugins mp
         JOIN org_marketplaces m ON m.id = mp.marketplace_id
         ORDER BY m.name",
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for row in rows {
        map.entry(row.plugin_id)
            .or_default()
            .push((row.id, row.name));
    }
    Ok(map)
}

pub async fn list_github_marketplaces(pool: &PgPool) -> Result<Vec<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as::<_, OrgMarketplace>(
        "SELECT id, name, description, github_repo_url, enabled, created_at, updated_at
         FROM org_marketplaces
         WHERE github_repo_url IS NOT NULL AND github_repo_url != ''
         ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, Copy)]
pub struct SyncLogEntry<'a> {
    pub marketplace_id: &'a str,
    pub operation: &'a str,
    pub status: &'a str,
    pub commit_hash: Option<&'a str>,
    pub plugins_synced: i64,
    pub errors: i64,
    pub error_message: Option<&'a str>,
    pub triggered_by: &'a str,
    pub duration_ms: Option<i64>,
}

pub async fn insert_sync_log(pool: &PgPool, entry: &SyncLogEntry<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO org_marketplace_sync_logs (marketplace_id, operation, status, commit_hash, plugins_synced, errors, error_message, triggered_by, duration_ms)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        entry.marketplace_id,
        entry.operation,
        entry.status,
        entry.commit_hash,
        entry.plugins_synced,
        entry.errors,
        entry.error_message,
        entry.triggered_by,
        entry.duration_ms,
    )
    .execute(pool)
    .await?;
    Ok(())
}
