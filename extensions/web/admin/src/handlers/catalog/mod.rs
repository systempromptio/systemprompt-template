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

mod view;

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use crate::handlers::shared;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{
    MarketplaceContext, McpServerDetail, PluginDetail, SkillCatalogEntry, UserContext,
    ENTITY_AGENT, ENTITY_MCP_SERVER, ENTITY_PLUGIN, ENTITY_SKILL,
};

use super::ssr::ssr_helpers::render_typed_page;
use view::{
    assignment_counts_by_type, build_row, forbidden, A2aPageData, CatalogRow, ExternalAgentRow,
    ExternalPageData, MarketplacePageData,
};

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

    let skills = skill_rows(raw_skills, &skill_counts);
    let plugins = plugin_rows(raw_plugins, &plugin_counts);
    let mcp_servers = mcp_rows(raw_mcp, &mcp_counts);

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

fn skill_rows(raw: Vec<SkillCatalogEntry>, counts: &HashMap<String, i64>) -> Vec<CatalogRow> {
    raw.into_iter()
        .map(|s| {
            let id_str = s.id.as_str().to_string();
            let count = counts.get(&id_str).copied().unwrap_or(0);
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
        .collect()
}

fn plugin_rows(raw: Vec<PluginDetail>, counts: &HashMap<String, i64>) -> Vec<CatalogRow> {
    raw.into_iter()
        .map(|p| {
            let count = counts.get(&p.id).copied().unwrap_or(0);
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
        .collect()
}

fn mcp_rows(raw: Vec<McpServerDetail>, counts: &HashMap<String, i64>) -> Vec<CatalogRow> {
    raw.into_iter()
        .map(|m| {
            let id_str = m.id.as_str().to_string();
            let count = counts.get(&id_str).copied().unwrap_or(0);
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
        .collect()
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
