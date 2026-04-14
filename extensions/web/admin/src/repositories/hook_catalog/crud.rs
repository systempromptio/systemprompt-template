use sqlx::PgPool;
use std::path::Path;

use super::super::super::types::{CreateHookRequest, HookCatalogEntry, UpdateHookRequest};
use super::scan::compute_checksum;
use super::{HookCatalogError, CATEGORY_CUSTOM, CATEGORY_SYSTEM, DEFAULT_VERSION};

pub async fn get_catalog_hook(
    pool: &PgPool,
    hook_id: &str,
) -> Result<Option<HookCatalogEntry>, HookCatalogError> {
    let row = sqlx::query_as!(HookCatalogEntry, "SELECT * FROM hook_catalog WHERE id = $1", hook_id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(mut hook) => {
            hook.plugins = sqlx::query_scalar!(
                "SELECT plugin_id FROM hook_plugins WHERE hook_id = $1 ORDER BY sort_order",
                hook.id,
            )
            .fetch_all(pool)
            .await?;
            Ok(Some(hook))
        }
        None => Ok(None),
    }
}

pub async fn list_catalog_hooks(pool: &PgPool) -> Result<Vec<HookCatalogEntry>, HookCatalogError> {
    let mut hooks = sqlx::query_as!(
        HookCatalogEntry,
        "SELECT * FROM hook_catalog ORDER BY category, event, id",
    )
    .fetch_all(pool)
    .await?;

    struct PluginRow {
        hook_id: String,
        plugin_id: String,
    }
    let plugin_rows_raw = sqlx::query_as!(
        PluginRow,
        "SELECT hook_id, plugin_id FROM hook_plugins ORDER BY hook_id, sort_order",
    )
    .fetch_all(pool)
    .await?;
    let plugin_rows: Vec<(String, String)> = plugin_rows_raw
        .into_iter()
        .map(|r| (r.hook_id, r.plugin_id))
        .collect();

    for hook in &mut hooks {
        hook.plugins = plugin_rows
            .iter()
            .filter(|(hid, _)| hid == &hook.id)
            .map(|(_, pid)| pid.clone())
            .collect();
    }

    Ok(hooks)
}

pub async fn create_catalog_hook(
    pool: &PgPool,
    services_path: &Path,
    req: &CreateHookRequest,
) -> Result<HookCatalogEntry, HookCatalogError> {
    let hook_id = generate_hook_id(&req.name, &req.event);
    let hooks_dir = services_path.join("hooks").join(&hook_id);
    std::fs::create_dir_all(&hooks_dir)?;

    let config_content = build_config_yaml(&ConfigYamlParams {
        hook_id: &hook_id,
        name: &req.name,
        description: &req.description,
        version: DEFAULT_VERSION,
        event: &req.event,
        matcher: &req.matcher,
        command: &req.command,
        is_async: req.is_async,
    });

    let config_path = hooks_dir.join("config.yaml");
    std::fs::write(&config_path, &config_content)?;

    let checksum = compute_checksum(&config_content);
    let now = chrono::Utc::now();

    sqlx::query!(
        r"INSERT INTO hook_catalog (id, name, description, version, event, matcher, command, is_async, category, enabled, tags, visible_to, checksum)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'custom', true, '{}', '{}', $9)",
        hook_id,
        req.name,
        req.description,
        DEFAULT_VERSION,
        req.event,
        req.matcher,
        req.command,
        req.is_async,
        checksum,
    )
    .execute(pool)
    .await?;

    if !req.plugin_id.is_empty() {
        sqlx::query!(
            "INSERT INTO hook_plugins (hook_id, plugin_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            hook_id,
            req.plugin_id,
        )
        .execute(pool)
        .await?;
    }

    Ok(HookCatalogEntry {
        id: hook_id,
        name: req.name.clone(),
        description: req.description.clone(),
        version: DEFAULT_VERSION.to_owned(),
        event: req.event.clone(),
        matcher: req.matcher.clone(),
        command: req.command.clone(),
        is_async: req.is_async,
        category: CATEGORY_CUSTOM.to_owned(),
        enabled: true,
        tags: vec![],
        visible_to: vec![],
        checksum,
        plugins: if req.plugin_id.is_empty() {
            vec![]
        } else {
            vec![req.plugin_id.clone()]
        },
        created_at: now,
        updated_at: now,
    })
}

pub async fn update_catalog_hook(
    pool: &PgPool,
    services_path: &Path,
    hook_id: &str,
    req: &UpdateHookRequest,
) -> Result<Option<HookCatalogEntry>, HookCatalogError> {
    let Some(current) = get_catalog_hook(pool, hook_id).await? else {
        return Ok(None);
    };

    if current.category == CATEGORY_SYSTEM {
        return Err(HookCatalogError::SystemHookModification);
    }

    let name = req.name.clone().unwrap_or(current.name);
    let description = req.description.clone().unwrap_or(current.description);
    let event = req.event.clone().unwrap_or(current.event);
    let matcher = req.matcher.clone().unwrap_or(current.matcher);
    let command = req.command.clone().unwrap_or(current.command);
    let is_async = req.is_async.unwrap_or(current.is_async);

    let hook_dir = services_path.join("hooks").join(hook_id);
    if hook_dir.exists() {
        let config_content = build_config_yaml(&ConfigYamlParams {
            hook_id,
            name: &name,
            description: &description,
            version: &current.version,
            event: &event,
            matcher: &matcher,
            command: &command,
            is_async,
        });
        let config_path = hook_dir.join("config.yaml");
        std::fs::write(&config_path, &config_content)?;
    }

    sqlx::query!(
        r"UPDATE hook_catalog SET name = $2, description = $3, event = $4, matcher = $5, command = $6, is_async = $7, updated_at = NOW()
          WHERE id = $1",
        hook_id,
        name,
        description,
        event,
        matcher,
        command,
        is_async,
    )
    .execute(pool)
    .await?;

    if let Some(ref plugin_id) = req.plugin_id {
        sqlx::query!("DELETE FROM hook_plugins WHERE hook_id = $1", hook_id)
            .execute(pool)
            .await?;
        if !plugin_id.is_empty() {
            sqlx::query!(
                "INSERT INTO hook_plugins (hook_id, plugin_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                hook_id,
                plugin_id,
            )
            .execute(pool)
            .await?;
        }
    }

    get_catalog_hook(pool, hook_id).await
}

pub async fn delete_catalog_hook(
    pool: &PgPool,
    services_path: &Path,
    hook_id: &str,
) -> Result<bool, HookCatalogError> {
    let Some(current) = get_catalog_hook(pool, hook_id).await? else {
        return Ok(false);
    };

    if current.category == CATEGORY_SYSTEM {
        return Err(HookCatalogError::SystemHookModification);
    }

    sqlx::query!("DELETE FROM hook_catalog WHERE id = $1", hook_id)
        .execute(pool)
        .await?;

    let hook_dir = services_path.join("hooks").join(hook_id);
    if hook_dir.exists() {
        std::fs::remove_dir_all(&hook_dir)?;
    }

    Ok(true)
}

fn generate_hook_id(name: &str, event: &str) -> String {
    let slug = name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");
    if slug.is_empty() {
        let uuid_str = uuid::Uuid::new_v4().to_string();
        let prefix = uuid_str.split('-').next().unwrap_or("0");
        format!("hook_{}_{}", event.to_lowercase(), prefix)
    } else {
        slug
    }
}

struct ConfigYamlParams<'a> {
    hook_id: &'a str,
    name: &'a str,
    description: &'a str,
    version: &'a str,
    event: &'a str,
    matcher: &'a str,
    command: &'a str,
    is_async: bool,
}

fn build_config_yaml(params: &ConfigYamlParams<'_>) -> String {
    format!(
        r#"id: {hook_id}
name: "{name}"
description: "{description}"
version: "{version}"
enabled: true
event: {event}
matcher: "{matcher}"
command: "{command}"
async: {is_async}
category: custom
tags: []
visible_to: []
"#,
        hook_id = params.hook_id,
        name = params.name.replace('"', "\\\""),
        description = params.description.replace('"', "\\\""),
        version = params.version,
        event = params.event,
        matcher = params.matcher,
        command = params.command.replace('"', "\\\""),
        is_async = params.is_async,
    )
}
