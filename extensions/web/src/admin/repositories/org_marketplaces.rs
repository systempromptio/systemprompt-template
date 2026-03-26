use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::marketplaces::{
    CreateOrgMarketplaceRequest, GitHubSyncLogEntry, OrgMarketplace, UpdateOrgMarketplaceRequest,
};

pub async fn list_org_marketplaces(pool: &Arc<PgPool>) -> Result<Vec<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as::<_, OrgMarketplace>(
        "SELECT id, name, description, enabled, github_repo_url, created_at, updated_at
         FROM org_marketplaces
         ORDER BY name",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_org_marketplace(
    pool: &Arc<PgPool>,
    id: &str,
) -> Result<Option<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as::<_, OrgMarketplace>(
        "SELECT id, name, description, enabled, github_repo_url, created_at, updated_at
         FROM org_marketplaces
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn create_org_marketplace(
    pool: &Arc<PgPool>,
    req: &CreateOrgMarketplaceRequest,
) -> Result<OrgMarketplace, sqlx::Error> {
    let marketplace = sqlx::query_as::<_, OrgMarketplace>(
        "INSERT INTO org_marketplaces (id, name, description, enabled, github_repo_url)
         VALUES ($1, $2, $3, true, $4)
         RETURNING id, name, description, enabled, github_repo_url, created_at, updated_at",
    )
    .bind(&req.id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.github_repo_url)
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
    let Some(existing) = get_org_marketplace(pool, id).await? else {
        return Ok(None);
    };

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let description = req.description.as_deref().unwrap_or(&existing.description);
    let github_repo_url = match &req.github_repo_url {
        Some(val) => val.as_deref(),
        None => existing.github_repo_url.as_deref(),
    };

    let updated = sqlx::query_as::<_, OrgMarketplace>(
        "UPDATE org_marketplaces
         SET name = $2, description = $3, github_repo_url = $4, updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, description, enabled, github_repo_url, created_at, updated_at",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(github_repo_url)
    .fetch_one(pool.as_ref())
    .await?;

    if let Some(ref plugin_ids) = req.plugin_ids {
        set_marketplace_plugins(pool, id, plugin_ids).await?;
    }

    Ok(Some(updated))
}

pub async fn delete_org_marketplace(pool: &Arc<PgPool>, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM org_marketplaces WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_marketplace_plugin_ids(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT plugin_id FROM org_marketplace_plugins
         WHERE marketplace_id = $1
         ORDER BY position, created_at",
    )
    .bind(marketplace_id)
    .fetch_all(pool.as_ref())
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn set_marketplace_plugins(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    plugin_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM org_marketplace_plugins WHERE marketplace_id = $1")
        .bind(marketplace_id)
        .execute(&mut *tx)
        .await?;

    for (i, plugin_id) in plugin_ids.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let position = i as i32;
        sqlx::query(
            "INSERT INTO org_marketplace_plugins (marketplace_id, plugin_id, position)
             VALUES ($1, $2, $3)",
        )
        .bind(marketplace_id)
        .bind(plugin_id)
        .bind(position)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn resolve_authorized_org_plugin_ids(
    pool: &Arc<PgPool>,
    user_roles: &[String],
    user_department: &str,
    is_admin: bool,
) -> Result<HashSet<String>, sqlx::Error> {
    let marketplaces = list_org_marketplaces(pool).await?;
    let all_rules = super::access_control::list_all_rules(pool).await?;

    let mut accessible_marketplace_ids: HashSet<String> = HashSet::new();

    for mkt in &marketplaces {
        let mkt_rules: Vec<_> = all_rules
            .iter()
            .filter(|r| r.entity_type == "marketplace" && r.entity_id == mkt.id)
            .collect();

        if mkt_rules.is_empty() {
            accessible_marketplace_ids.insert(mkt.id.clone());
            continue;
        }

        let has_role_match = mkt_rules.iter().any(|r| {
            r.rule_type == "role" && r.access == "allow" && user_roles.contains(&r.rule_value)
        });

        let has_dept_match = mkt_rules.iter().any(|r| {
            r.rule_type == "department" && r.access == "allow" && r.rule_value == user_department
        });

        if is_admin || has_role_match || has_dept_match {
            accessible_marketplace_ids.insert(mkt.id.clone());
        }
    }

    let mut org_plugin_ids: HashSet<String> = HashSet::new();
    for mkt in &marketplaces {
        if !accessible_marketplace_ids.contains(&mkt.id) {
            continue;
        }
        if let Ok(plugin_ids) = list_marketplace_plugin_ids(pool, &mkt.id).await {
            for pid in plugin_ids {
                org_plugin_ids.insert(pid);
            }
        }
    }

    Ok(org_plugin_ids)
}

pub async fn resolve_authorized_marketplace_groups(
    pool: &Arc<PgPool>,
    user_roles: &[String],
    user_department: &str,
    is_admin: bool,
) -> Result<Vec<(OrgMarketplace, Vec<String>)>, sqlx::Error> {
    let marketplaces = list_org_marketplaces(pool).await?;
    let all_rules = super::access_control::list_all_rules(pool).await?;

    let mut result = Vec::new();

    for mkt in marketplaces {
        let mkt_rules: Vec<_> = all_rules
            .iter()
            .filter(|r| r.entity_type == "marketplace" && r.entity_id == mkt.id)
            .collect();

        let accessible = if mkt_rules.is_empty() {
            true
        } else {
            let has_role_match = mkt_rules.iter().any(|r| {
                r.rule_type == "role" && r.access == "allow" && user_roles.contains(&r.rule_value)
            });
            let has_dept_match = mkt_rules.iter().any(|r| {
                r.rule_type == "department"
                    && r.access == "allow"
                    && r.rule_value == user_department
            });
            is_admin || has_role_match || has_dept_match
        };

        if accessible {
            let plugin_ids = list_marketplace_plugin_ids(pool, &mkt.id)
                .await
                .unwrap_or_default();
            result.push((mkt, plugin_ids));
        }
    }

    Ok(result)
}

pub async fn list_marketplaces_for_plugins(
    pool: &Arc<PgPool>,
) -> Result<HashMap<String, Vec<(String, String)>>, sqlx::Error> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT mp.plugin_id, m.id, m.name
         FROM org_marketplace_plugins mp
         JOIN org_marketplaces m ON m.id = mp.marketplace_id
         ORDER BY m.name",
    )
    .fetch_all(pool.as_ref())
    .await?;

    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (plugin_id, mkt_id, mkt_name) in rows {
        map.entry(plugin_id).or_default().push((mkt_id, mkt_name));
    }
    Ok(map)
}

pub async fn list_github_marketplaces(
    pool: &Arc<PgPool>,
) -> Result<Vec<OrgMarketplace>, sqlx::Error> {
    sqlx::query_as::<_, OrgMarketplace>(
        "SELECT id, name, description, enabled, github_repo_url, created_at, updated_at
         FROM org_marketplaces
         WHERE github_repo_url IS NOT NULL AND enabled = true
         ORDER BY name",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn insert_sync_log(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
    action: &str,
    status: &str,
    commit_hash: Option<&str>,
    plugin_count: i32,
    error_count: i32,
    error_message: Option<&str>,
    triggered_by: &str,
    duration_ms: Option<i64>,
) -> Result<GitHubSyncLogEntry, sqlx::Error> {
    sqlx::query_as::<_, GitHubSyncLogEntry>(
        "INSERT INTO github_marketplace_sync_log
         (marketplace_id, action, status, commit_hash, plugin_count, error_count, error_message, triggered_by, duration_ms)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, marketplace_id, action, status, commit_hash, plugin_count, error_count, error_message, triggered_by, duration_ms, created_at",
    )
    .bind(marketplace_id)
    .bind(action)
    .bind(status)
    .bind(commit_hash)
    .bind(plugin_count)
    .bind(error_count)
    .bind(error_message)
    .bind(triggered_by)
    .bind(duration_ms)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn get_latest_sync(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
) -> Result<Option<GitHubSyncLogEntry>, sqlx::Error> {
    sqlx::query_as::<_, GitHubSyncLogEntry>(
        "SELECT id, marketplace_id, action, status, commit_hash, plugin_count, error_count, error_message, triggered_by, duration_ms, created_at
         FROM github_marketplace_sync_log
         WHERE marketplace_id = $1 AND status = 'success'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(marketplace_id)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn count_users_with_marketplace_access(
    pool: &Arc<PgPool>,
    marketplace_id: &str,
) -> Result<i64, sqlx::Error> {
    let rules_exist: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM access_control_rules WHERE entity_type = 'marketplace' AND entity_id = $1)",
    )
    .bind(marketplace_id)
    .fetch_one(pool.as_ref())
    .await?;

    if !rules_exist {
        // No rules means all users have access
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE status = 'active'")
            .fetch_one(pool.as_ref())
            .await?;
        return Ok(count);
    }

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT u.id) FROM users u
         WHERE u.status = 'active' AND (
             EXISTS (
                 SELECT 1 FROM access_control_rules acr
                 WHERE acr.entity_type = 'marketplace' AND acr.entity_id = $1
                 AND acr.access = 'allow' AND acr.rule_type = 'role'
                 AND acr.rule_value = ANY(u.roles)
             )
             OR EXISTS (
                 SELECT 1 FROM access_control_rules acr
                 WHERE acr.entity_type = 'marketplace' AND acr.entity_id = $1
                 AND acr.access = 'allow' AND acr.rule_type = 'department'
                 AND acr.rule_value = u.department
             )
         )",
    )
    .bind(marketplace_id)
    .fetch_one(pool.as_ref())
    .await?;

    Ok(count)
}
