mod builders;

use std::collections::HashMap;
use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::access_control::AccessControlRule;
use crate::types::{DepartmentStats, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;
use builders::{build_agents_json, build_departments_json, build_mcp_json, build_plugins_json};

struct AccessControlData {
    all_rules: Vec<AccessControlRule>,
    departments: Vec<DepartmentStats>,
}

fn load_filesystem_entities(
    services_path: &std::path::Path,
) -> (
    Vec<crate::types::PluginOverview>,
    Vec<crate::types::AgentDetail>,
    Vec<crate::types::McpServerDetail>,
) {
    let admin_roles = vec!["admin".to_string()];
    let plugins =
        repositories::list_plugins_for_roles(services_path, &admin_roles).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins");
            vec![]
        });

    let agents = repositories::list_agents(services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list agents");
        vec![]
    });

    let mcp_servers =
        repositories::mcp_servers::list_mcp_servers(services_path).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list MCP servers");
            vec![]
        });

    (plugins, agents, mcp_servers)
}

async fn load_access_control_data(pool: &PgPool) -> AccessControlData {
    let (rules_res, dept_res) = tokio::join!(
        repositories::access_control::list_all_rules(pool),
        repositories::fetch_department_stats(pool),
    );

    let all_rules = rules_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch access control rules");
        vec![]
    });

    let departments = dept_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch department stats");
        vec![]
    });

    AccessControlData {
        all_rules,
        departments,
    }
}

fn build_rules_map(
    all_rules: &[AccessControlRule],
) -> HashMap<(String, String), Vec<&AccessControlRule>> {
    let mut rules_map: HashMap<(String, String), Vec<&AccessControlRule>> = HashMap::new();
    for rule in all_rules {
        rules_map
            .entry((rule.entity_type.clone(), rule.entity_id.clone()))
            .or_default()
            .push(rule);
    }
    rules_map
}

pub async fn access_control_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let (plugins, agents, mcp_servers) = load_filesystem_entities(&services_path);
    let ac_data = load_access_control_data(&pool).await;
    let data = assemble_page_data(&services_path, &plugins, &agents, &mcp_servers, &ac_data);

    super::render_page(&engine, "access-control", &data, &user_ctx, &mkt_ctx)
}

fn assemble_page_data(
    services_path: &std::path::Path,
    plugins: &[crate::types::PluginOverview],
    agents: &[crate::types::AgentDetail],
    mcp_servers: &[crate::types::McpServerDetail],
    ac_data: &AccessControlData,
) -> serde_json::Value {
    let rules_map = build_rules_map(&ac_data.all_rules);
    let known_roles = vec!["admin", "developer", "analyst", "viewer"];
    let dept_names: Vec<&str> = ac_data
        .departments
        .iter()
        .map(|d| d.department.as_str())
        .collect();

    let plugins_json = build_plugins_json(
        services_path,
        plugins,
        &rules_map,
        &known_roles,
        &dept_names,
        &ac_data.departments,
    );
    let agents_json = build_agents_json(
        agents,
        &rules_map,
        &known_roles,
        &dept_names,
        &ac_data.departments,
    );
    let mcp_json = build_mcp_json(
        mcp_servers,
        &rules_map,
        &known_roles,
        &dept_names,
        &ac_data.departments,
    );

    json!({
        "page": "access-control",
        "title": "Access Control",
        "plugins": plugins_json,
        "agents": agents_json,
        "mcp_servers": mcp_json,
        "departments": build_departments_json(&ac_data.departments),
        "known_roles": known_roles,
        "stats": {
            "plugin_count": plugins.len(),
            "agent_count": agents.len(),
            "mcp_count": mcp_servers.len(),
            "department_count": ac_data.departments.len(),
        },
    })
}
