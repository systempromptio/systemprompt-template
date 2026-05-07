//! Unified read-only `/admin/catalog` page.
//!
//! Aggregates skills, plugins, MCP servers and agents from the YAML services
//! tree, augments each row with assignment counts pulled from
//! `access_control_rules`, and renders the `catalog` Handlebars template.
//!
//! This page is strictly read-only: there are no POST/PUT/DELETE companion
//! routes. Operators edit `services/*.yaml` and restart; the dashboard is a
//! viewer.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::handlers::shared;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{
    MarketplaceContext, UserContext, ENTITY_AGENT, ENTITY_MCP_SERVER, ENTITY_PLUGIN, ENTITY_SKILL,
};

use super::ssr::ssr_helpers::render_typed_page;

#[derive(Debug, Serialize)]
struct CatalogRow {
    entity_type: &'static str,
    id: String,
    name: String,
    description: String,
    enabled: bool,
    source_path: String,
    assignment_count: i64,
    matrix_url: String,
}

#[derive(Debug, Serialize)]
struct CatalogPageData {
    page: &'static str,
    title: &'static str,
    skills: Vec<CatalogRow>,
    plugins: Vec<CatalogRow>,
    mcp_servers: Vec<CatalogRow>,
    agents: Vec<CatalogRow>,
    skills_count: usize,
    plugins_count: usize,
    mcp_servers_count: usize,
    agents_count: usize,
    total: usize,
    types: Vec<&'static str>,
}

fn matrix_url(entity_type: &str, entity_id: &str) -> String {
    format!("/admin/access/matrix?entity_type={entity_type}&entity_id={entity_id}")
}

async fn assignment_counts_by_type(
    pool: &PgPool,
    entity_type: &str,
) -> HashMap<String, i64> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT entity_id, COUNT(*)::BIGINT
         FROM access_control_rules
         WHERE entity_type = $1
         GROUP BY entity_id",
    )
    .bind(entity_type)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, entity_type, "Failed to load assignment counts");
        Vec::new()
    });
    rows.into_iter().collect()
}

fn build_row(
    entity_type: &'static str,
    id: String,
    name: String,
    description: String,
    enabled: bool,
    source_path: String,
    assignment_count: i64,
) -> CatalogRow {
    CatalogRow {
        entity_type,
        matrix_url: matrix_url(entity_type, &id),
        id,
        name,
        description,
        enabled,
        source_path,
        assignment_count,
    }
}

struct RawCatalog {
    skills: Vec<crate::types::SkillCatalogEntry>,
    plugins: Vec<crate::types::PluginDetail>,
    mcp_servers: Vec<crate::types::McpServerDetail>,
    agents: Vec<crate::types::AgentCatalogEntry>,
}

fn load_raw_catalog(services_path: &std::path::Path) -> RawCatalog {
    let skills = repositories::list_skill_catalog(services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load skill catalog");
        Vec::new()
    });
    let plugins = repositories::list_plugin_catalog(services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load plugin catalog");
        Vec::new()
    });
    let mcp_servers = repositories::mcp_servers::list_mcp_servers(services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load MCP catalog");
        Vec::new()
    });
    let agents = repositories::list_agent_catalog(services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load agent catalog");
        Vec::new()
    });
    RawCatalog {
        skills,
        plugins,
        mcp_servers,
        agents,
    }
}

pub async fn catalog_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Html("<h1>Access Denied</h1><p>Admin access required.</p>"),
        )
            .into_response();
    }

    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let raw = load_raw_catalog(&services_path);
    let (skills, plugins, mcp_servers, agents) = build_catalog_sections(&pool, raw).await;

    let skills_count = skills.len();
    let plugins_count = plugins.len();
    let mcp_servers_count = mcp_servers.len();
    let agents_count = agents.len();
    let total = skills_count + plugins_count + mcp_servers_count + agents_count;

    let data = CatalogPageData {
        page: "catalog",
        title: "Catalog",
        types: vec![ENTITY_SKILL, ENTITY_PLUGIN, ENTITY_MCP_SERVER, ENTITY_AGENT],
        skills,
        plugins,
        mcp_servers,
        agents,
        skills_count,
        plugins_count,
        mcp_servers_count,
        agents_count,
        total,
    };

    render_typed_page(&engine, "catalog", &data, &user_ctx, &mkt_ctx)
}

async fn build_catalog_sections(
    pool: &PgPool,
    raw: RawCatalog,
) -> (
    Vec<CatalogRow>,
    Vec<CatalogRow>,
    Vec<CatalogRow>,
    Vec<CatalogRow>,
) {
    let (skill_counts, plugin_counts, mcp_counts, agent_counts) = tokio::join!(
        assignment_counts_by_type(pool, ENTITY_SKILL),
        assignment_counts_by_type(pool, ENTITY_PLUGIN),
        assignment_counts_by_type(pool, ENTITY_MCP_SERVER),
        assignment_counts_by_type(pool, ENTITY_AGENT),
    );

    let skills = raw
        .skills
        .into_iter()
        .map(|s| {
            let id_str = s.id.as_str().to_string();
            let count = skill_counts.get(&id_str).copied().unwrap_or(0);
            build_row(
                ENTITY_SKILL,
                id_str,
                s.name,
                s.description,
                s.enabled,
                s.source_path,
                count,
            )
        })
        .collect();

    let plugins = raw
        .plugins
        .into_iter()
        .map(|p| {
            let count = plugin_counts.get(&p.id).copied().unwrap_or(0);
            build_row(
                ENTITY_PLUGIN,
                p.id,
                p.name,
                p.description,
                p.enabled,
                p.source_path,
                count,
            )
        })
        .collect();

    let mcp_servers = raw
        .mcp_servers
        .into_iter()
        .map(|m| {
            let id_str = m.id.as_str().to_string();
            let count = mcp_counts.get(&id_str).copied().unwrap_or(0);
            build_row(
                ENTITY_MCP_SERVER,
                id_str.clone(),
                id_str,
                m.description,
                m.enabled,
                m.source_path,
                count,
            )
        })
        .collect();

    let agents = raw
        .agents
        .into_iter()
        .map(|a| {
            let id_str = a.id.as_str().to_string();
            let count = agent_counts.get(&id_str).copied().unwrap_or(0);
            build_row(
                ENTITY_AGENT,
                id_str,
                a.name,
                a.description,
                a.enabled,
                a.source_path,
                count,
            )
        })
        .collect();

    (skills, plugins, mcp_servers, agents)
}
