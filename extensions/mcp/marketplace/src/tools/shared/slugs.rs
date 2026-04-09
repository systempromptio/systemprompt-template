use sqlx::PgPool;
use std::collections::HashMap;
use systemprompt::mcp::McpError;

pub async fn resolve_skill_slugs(
    pool: &PgPool,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT skill_id, id FROM user_skills WHERE user_id = $1 AND skill_id = ANY($2)",
    )
    .bind(user_id)
    .bind(slugs)
    .fetch_all(pool)
    .await
    .map_err(|e| McpError::internal_error(format!("Failed to resolve Skill slugs: {e}"), None))?;

    let map: HashMap<String, String> = rows.into_iter().collect();
    slugs
        .iter()
        .map(|slug| {
            map.get(slug)
                .cloned()
                .ok_or_else(|| McpError::invalid_params(format!("Skill '{slug}' not found"), None))
        })
        .collect()
}

pub async fn resolve_agent_slugs(
    pool: &PgPool,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT agent_id, id FROM user_agents WHERE user_id = $1 AND agent_id = ANY($2)",
    )
    .bind(user_id)
    .bind(slugs)
    .fetch_all(pool)
    .await
    .map_err(|e| McpError::internal_error(format!("Failed to resolve Agent slugs: {e}"), None))?;

    let map: HashMap<String, String> = rows.into_iter().collect();
    slugs
        .iter()
        .map(|slug| {
            map.get(slug)
                .cloned()
                .ok_or_else(|| McpError::invalid_params(format!("Agent '{slug}' not found"), None))
        })
        .collect()
}

pub async fn resolve_mcp_server_slugs(
    pool: &PgPool,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT mcp_server_id, id FROM user_mcp_servers WHERE user_id = $1 AND mcp_server_id = ANY($2)")
            .bind(user_id)
            .bind(slugs)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to resolve MCP server slugs: {e}"),
                    None,
                )
            })?;

    let map: HashMap<String, String> = rows.into_iter().collect();
    slugs
        .iter()
        .map(|slug| {
            map.get(slug).cloned().ok_or_else(|| {
                McpError::invalid_params(format!("MCP server '{slug}' not found"), None)
            })
        })
        .collect()
}

pub async fn resolve_skill_uuids_to_slugs(
    pool: &PgPool,
    uuids: &[String],
) -> Result<Vec<String>, McpError> {
    if uuids.is_empty() {
        return Ok(vec![]);
    }

    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, skill_id FROM user_skills WHERE id = ANY($1)")
            .bind(uuids)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to resolve Skill UUIDs: {e}"), None)
            })?;

    let map: HashMap<String, String> = rows.into_iter().collect();
    Ok(uuids
        .iter()
        .filter_map(|uuid| map.get(uuid).cloned())
        .collect())
}

pub async fn resolve_agent_uuids_to_slugs(
    pool: &PgPool,
    uuids: &[String],
) -> Result<Vec<String>, McpError> {
    if uuids.is_empty() {
        return Ok(vec![]);
    }

    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, agent_id FROM user_agents WHERE id = ANY($1)")
            .bind(uuids)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to resolve Agent UUIDs: {e}"), None)
            })?;

    let map: HashMap<String, String> = rows.into_iter().collect();
    Ok(uuids
        .iter()
        .filter_map(|uuid| map.get(uuid).cloned())
        .collect())
}

pub async fn resolve_mcp_server_uuids_to_slugs(
    pool: &PgPool,
    uuids: &[String],
) -> Result<Vec<String>, McpError> {
    if uuids.is_empty() {
        return Ok(vec![]);
    }

    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, mcp_server_id FROM user_mcp_servers WHERE id = ANY($1)")
            .bind(uuids)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to resolve MCP server UUIDs: {e}"), None)
            })?;

    let map: HashMap<String, String> = rows.into_iter().collect();
    Ok(uuids
        .iter()
        .filter_map(|uuid| map.get(uuid).cloned())
        .collect())
}
