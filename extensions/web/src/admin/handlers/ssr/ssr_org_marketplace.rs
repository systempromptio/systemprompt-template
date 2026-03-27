use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

pub(crate) async fn org_marketplace_page(
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

    let marketplaces = match repositories::org_marketplaces::list_org_marketplaces(&pool).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list org marketplaces");
            vec![]
        }
    };

    let all_rules = repositories::access_control::list_all_rules(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch access control rules");
            vec![]
        });

    let mut rules_map: HashMap<
        (String, String),
        Vec<&crate::admin::types::access_control::AccessControlRule>,
    > = HashMap::new();
    for rule in &all_rules {
        if rule.entity_type == "marketplace" {
            rules_map
                .entry((rule.entity_type.clone(), rule.entity_id.clone()))
                .or_default()
                .push(rule);
        }
    }

    let known_roles = repositories::fetch_distinct_roles(&pool)
        .await
        .unwrap_or_default();

    let all_roles_json: Vec<serde_json::Value> = known_roles
        .iter()
        .map(|r| json!({ "value": r }))
        .collect();

    let all_assoc = sqlx::query!(
        "SELECT marketplace_id, plugin_id
         FROM org_marketplace_plugins
         ORDER BY marketplace_id, position, created_at",
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to list marketplace plugin associations");
        vec![]
    });

    let mut plugin_map: HashMap<String, Vec<String>> = HashMap::new();
    for row in all_assoc {
        plugin_map
            .entry(row.marketplace_id)
            .or_default()
            .push(row.plugin_id);
    }

    let mut marketplaces_json: Vec<serde_json::Value> = Vec::new();
    let mut total_plugins = 0usize;
    let mut total_skills = 0usize;

    for mkt in &marketplaces {
        let plugin_ids = plugin_map.remove(mkt.id.as_str()).unwrap_or_default();

        let mut skill_count = 0usize;
        let mut agent_count = 0usize;
        let mut mcp_count = 0usize;
        let mut plugins_detail: Vec<serde_json::Value> = Vec::new();

        for pid in &plugin_ids {
            if let Ok(Some(detail)) = repositories::find_plugin_detail(&services_path, pid) {
                let sc = detail.skills.len();
                let ac = detail.agents.len();
                let mc = detail.mcp_servers.len();
                skill_count += sc;
                agent_count += ac;
                mcp_count += mc;
                plugins_detail.push(json!({
                    "id": detail.id,
                    "name": detail.name,
                    "description": detail.description,
                    "category": detail.category,
                    "skill_count": sc,
                    "agent_count": ac,
                    "mcp_count": mc,
                }));
            } else {
                plugins_detail.push(json!({
                    "id": pid,
                    "name": pid,
                    "description": "",
                    "category": "",
                    "skill_count": 0,
                    "agent_count": 0,
                    "mcp_count": 0,
                }));
            }
        }

        total_plugins += plugin_ids.len();
        total_skills += skill_count;

        let key = ("marketplace".to_string(), mkt.id.clone());
        let entity_rules = rules_map.get(&key);

        let mut role_count = 0;
        let roles: Vec<serde_json::Value> = known_roles
            .iter()
            .map(|role_name| {
                let assigned = entity_rules.is_some_and(|rules| {
                    rules.iter().any(|r| {
                        r.rule_type == "role" && r.rule_value == *role_name && r.access == "allow"
                    })
                });
                if assigned {
                    role_count += 1;
                }
                json!({ "name": role_name, "assigned": assigned })
            })
            .collect();

        marketplaces_json.push(json!({
            "id": mkt.id,
            "name": mkt.name,
            "description": mkt.description,
            "created_at": mkt.created_at.to_rfc3339(),
            "updated_at": mkt.updated_at.to_rfc3339(),
            "plugin_count": plugin_ids.len(),
            "skill_count": skill_count,
            "agent_count": agent_count,
            "mcp_count": mcp_count,
            "plugins": plugins_detail,
            "plugin_ids": plugin_ids,
            "roles": roles,
            "role_count": role_count,
        }));
    }

    // JSON: protocol boundary — template renders via json helper
    let data = json!({
        "page": "org-marketplace",
        "title": "Marketplace Management",
        "marketplaces": marketplaces_json,
        "all_roles": all_roles_json,
        "stats": {
            "marketplace_count": marketplaces.len(),
            "total_plugins": total_plugins,
            "total_skills": total_skills,
        },
    });

    super::render_page(&engine, "org-marketplace", &data, &user_ctx, &mkt_ctx)
}
