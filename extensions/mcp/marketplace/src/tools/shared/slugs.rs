use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::mcp::McpError;

async fn resolve_slugs_generic(
    pool: &Arc<PgPool>,
    user_id: &str,
    slugs: &[String],
    table: &str,
    slug_col: &str,
    id_col: &str,
    entity_label: &str,
) -> Result<Vec<String>, McpError> {
    if slugs.is_empty() {
        return Ok(vec![]);
    }

    let query = format!(
        "SELECT {slug_col}, {id_col} FROM {table} WHERE user_id = $1 AND {slug_col} = ANY($2)"
    );

    let rows: Vec<(String, String)> = sqlx::query_as(&query)
        .bind(user_id)
        .bind(slugs)
        .fetch_all(pool.as_ref())
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to resolve {entity_label} slugs: {e}"), None)
        })?;

    let map: std::collections::HashMap<String, String> = rows.into_iter().collect();
    slugs
        .iter()
        .map(|slug| {
            map.get(slug).cloned().ok_or_else(|| {
                McpError::invalid_params(format!("{entity_label} '{slug}' not found"), None)
            })
        })
        .collect()
}

async fn resolve_uuids_to_slugs_generic(
    pool: &Arc<PgPool>,
    uuids: &[String],
    table: &str,
    id_col: &str,
    slug_col: &str,
) -> Vec<String> {
    if uuids.is_empty() {
        return vec![];
    }

    let query = format!("SELECT {id_col}, {slug_col} FROM {table} WHERE {id_col} = ANY($1)");

    let rows: Vec<(String, String)> = sqlx::query_as(&query)
        .bind(uuids)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_default();

    let map: std::collections::HashMap<String, String> = rows.into_iter().collect();
    uuids
        .iter()
        .filter_map(|uuid| map.get(uuid).cloned())
        .collect()
}

pub async fn resolve_skill_slugs(
    pool: &Arc<PgPool>,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    resolve_slugs_generic(
        pool,
        user_id,
        slugs,
        "user_skills",
        "skill_id",
        "id",
        "Skill",
    )
    .await
}

pub async fn resolve_agent_slugs(
    pool: &Arc<PgPool>,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    resolve_slugs_generic(
        pool,
        user_id,
        slugs,
        "user_agents",
        "agent_id",
        "id",
        "Agent",
    )
    .await
}

pub async fn resolve_mcp_server_slugs(
    pool: &Arc<PgPool>,
    user_id: &str,
    slugs: &[String],
) -> Result<Vec<String>, McpError> {
    resolve_slugs_generic(
        pool,
        user_id,
        slugs,
        "user_mcp_servers",
        "mcp_server_id",
        "id",
        "MCP server",
    )
    .await
}

pub async fn resolve_skill_uuids_to_slugs(pool: &Arc<PgPool>, uuids: &[String]) -> Vec<String> {
    resolve_uuids_to_slugs_generic(pool, uuids, "user_skills", "id", "skill_id").await
}

pub async fn resolve_agent_uuids_to_slugs(pool: &Arc<PgPool>, uuids: &[String]) -> Vec<String> {
    resolve_uuids_to_slugs_generic(pool, uuids, "user_agents", "id", "agent_id").await
}

pub async fn resolve_mcp_server_uuids_to_slugs(
    pool: &Arc<PgPool>,
    uuids: &[String],
) -> Vec<String> {
    resolve_uuids_to_slugs_generic(pool, uuids, "user_mcp_servers", "id", "mcp_server_id").await
}
