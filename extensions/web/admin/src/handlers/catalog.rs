//! Read-only catalog admin pages, split into three views:
//!
//! - `/admin/catalog/marketplace` — install-able units (skills, plugins, MCP
//!   servers) loaded from `services/*.yaml`. Mirrors Anthropic's plugin
//!   marketplace mental model.
//! - `/admin/catalog/a2a` — A2A agents from `services/agents/*.yaml`. These
//!   run as standalone services and connect to the gateway as peers.
//! - `/admin/catalog/external` — external host apps from
//!   `services/external_agents/*.yaml` (Claude Desktop, Codex CLI). They
//!   connect via `systemprompt-bridge` and the `enabled` flag here mirrors
//!   what surfaces on `/admin/profile` under "Available agents".
//!
//! All three pages are strictly read-only: there are no POST/PUT/DELETE
//! companion routes. Operators edit `services/*.yaml` and restart.

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
struct MarketplacePageData {
    page: &'static str,
    title: &'static str,
    skills: Vec<CatalogRow>,
    plugins: Vec<CatalogRow>,
    mcp_servers: Vec<CatalogRow>,
    skills_count: usize,
    plugins_count: usize,
    mcp_servers_count: usize,
    total: usize,
    types: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct A2aPageData {
    page: &'static str,
    title: &'static str,
    agents: Vec<CatalogRow>,
    agents_count: usize,
}

#[derive(Debug, Serialize)]
struct ExternalAgentRow {
    id: String,
    display_name: String,
    kind: String,
    enabled: bool,
    description: String,
    platforms: Vec<String>,
    docs_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExternalPageData {
    page: &'static str,
    title: &'static str,
    agents: Vec<ExternalAgentRow>,
    agents_count: usize,
    enabled_count: usize,
}

fn matrix_url(entity_type: &str, entity_id: &str) -> String {
    format!("/admin/access/matrix?entity_type={entity_type}&entity_id={entity_id}")
}

async fn assignment_counts_by_type(pool: &PgPool, entity_type: &str) -> HashMap<String, i64> {
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

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Html("<h1>Access Denied</h1><p>Admin access required.</p>"),
    )
        .into_response()
}

pub async fn marketplace_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let raw_skills = repositories::list_skill_catalog(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load skill catalog");
        Vec::new()
    });
    let raw_plugins = repositories::list_plugin_catalog(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load plugin catalog");
        Vec::new()
    });
    let raw_mcp = repositories::mcp_servers::list_mcp_servers(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load MCP catalog");
        Vec::new()
    });

    let (skill_counts, plugin_counts, mcp_counts) = tokio::join!(
        assignment_counts_by_type(&pool, ENTITY_SKILL),
        assignment_counts_by_type(&pool, ENTITY_PLUGIN),
        assignment_counts_by_type(&pool, ENTITY_MCP_SERVER),
    );

    let skills: Vec<CatalogRow> = raw_skills
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

    let plugins: Vec<CatalogRow> = raw_plugins
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

    let mcp_servers: Vec<CatalogRow> = raw_mcp
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

    let skills_count = skills.len();
    let plugins_count = plugins.len();
    let mcp_servers_count = mcp_servers.len();
    let total = skills_count + plugins_count + mcp_servers_count;

    let data = MarketplacePageData {
        page: "marketplace",
        title: "Marketplace",
        types: vec![ENTITY_SKILL, ENTITY_PLUGIN, ENTITY_MCP_SERVER],
        skills,
        plugins,
        mcp_servers,
        skills_count,
        plugins_count,
        mcp_servers_count,
        total,
    };

    render_typed_page(&engine, "catalog-marketplace", &data, &user_ctx, &mkt_ctx)
}

pub async fn a2a_agents_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let raw_agents = repositories::list_agent_catalog(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load agent catalog");
        Vec::new()
    });
    let agent_counts = assignment_counts_by_type(&pool, ENTITY_AGENT).await;

    let agents: Vec<CatalogRow> = raw_agents
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

    let agents_count = agents.len();

    let data = A2aPageData {
        page: "a2a-agents",
        title: "A2A agents",
        agents,
        agents_count,
    };

    render_typed_page(&engine, "catalog-a2a", &data, &user_ctx, &mkt_ctx)
}

pub async fn external_agents_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let raw = repositories::external_agents_grp::list_external_agents();

    let agents: Vec<ExternalAgentRow> = raw
        .into_iter()
        .map(|e| ExternalAgentRow {
            id: e.id,
            display_name: e.display_name,
            kind: e.kind,
            enabled: e.enabled,
            description: e.description,
            platforms: e.platforms,
            docs_url: e.docs_url,
        })
        .collect();

    let agents_count = agents.len();
    let enabled_count = agents.iter().filter(|a| a.enabled).count();

    let data = ExternalPageData {
        page: "external-agents",
        title: "External agents",
        agents,
        agents_count,
        enabled_count,
    };

    render_typed_page(&engine, "catalog-external", &data, &user_ctx, &mkt_ctx)
}
