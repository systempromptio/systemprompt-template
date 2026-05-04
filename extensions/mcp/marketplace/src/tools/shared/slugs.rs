use std::collections::HashMap;
use systemprompt::mcp::McpError;

use crate::repositories::slugs as slug_repo;

pub async fn resolve_skill_slugs<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
    pool: E,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let raw_rows = slug_repo::list_user_skill_slugs(pool, user_id, slugs)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to resolve Skill slugs: {e}"), None)
        })?;
    let map: HashMap<String, String> = raw_rows.into_iter().map(|r| (r.skill_id, r.id)).collect();
    slugs
        .iter()
        .map(|slug| {
            map.get(slug)
                .cloned()
                .ok_or_else(|| McpError::invalid_params(format!("Skill '{slug}' not found"), None))
        })
        .collect()
}

pub async fn resolve_agent_slugs<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
    pool: E,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let raw_rows = slug_repo::list_user_agent_slugs(pool, user_id, slugs)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to resolve Agent slugs: {e}"), None)
        })?;
    let map: HashMap<String, String> = raw_rows.into_iter().map(|r| (r.agent_id, r.id)).collect();
    slugs
        .iter()
        .map(|slug| {
            map.get(slug)
                .cloned()
                .ok_or_else(|| McpError::invalid_params(format!("Agent '{slug}' not found"), None))
        })
        .collect()
}

pub async fn resolve_mcp_server_slugs<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
    pool: E,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let raw_rows = slug_repo::list_user_mcp_server_slugs(pool, user_id, slugs)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to resolve MCP server slugs: {e}"), None)
        })?;
    let map: HashMap<String, String> = raw_rows
        .into_iter()
        .map(|r| (r.mcp_server_id, r.id))
        .collect();
    slugs
        .iter()
        .map(|slug| {
            map.get(slug).cloned().ok_or_else(|| {
                McpError::invalid_params(format!("MCP server '{slug}' not found"), None)
            })
        })
        .collect()
}

pub async fn resolve_skill_uuids_to_slugs<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
    pool: E,
    uuids: &[String],
) -> Result<Vec<String>, McpError> {
    if uuids.is_empty() {
        return Ok(vec![]);
    }

    let raw_rows = slug_repo::list_user_skill_uuids(pool, uuids)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to resolve Skill UUIDs: {e}"), None)
        })?;
    let map: HashMap<String, String> = raw_rows.into_iter().map(|r| (r.id, r.slug)).collect();
    Ok(uuids
        .iter()
        .filter_map(|uuid| map.get(uuid).cloned())
        .collect())
}

pub async fn resolve_agent_uuids_to_slugs<'e, E: sqlx::Executor<'e, Database = sqlx::Postgres>>(
    pool: E,
    uuids: &[String],
) -> Result<Vec<String>, McpError> {
    if uuids.is_empty() {
        return Ok(vec![]);
    }

    let raw_rows = slug_repo::list_user_agent_uuids(pool, uuids)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to resolve Agent UUIDs: {e}"), None)
        })?;
    let map: HashMap<String, String> = raw_rows.into_iter().map(|r| (r.id, r.slug)).collect();
    Ok(uuids
        .iter()
        .filter_map(|uuid| map.get(uuid).cloned())
        .collect())
}

pub async fn resolve_mcp_server_uuids_to_slugs<
    'e,
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
>(
    pool: E,
    uuids: &[String],
) -> Result<Vec<String>, McpError> {
    if uuids.is_empty() {
        return Ok(vec![]);
    }

    let raw_rows = slug_repo::list_user_mcp_server_uuids(pool, uuids)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to resolve MCP server UUIDs: {e}"), None)
        })?;
    let map: HashMap<String, String> = raw_rows.into_iter().map(|r| (r.id, r.slug)).collect();
    Ok(uuids
        .iter()
        .filter_map(|uuid| map.get(uuid).cloned())
        .collect())
}
