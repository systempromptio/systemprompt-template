use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, PluginDetail, PluginOverview, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn plugins_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let enabled_map = repositories::get_agent_skills_enabled_map(&pool)
        .await
        .unwrap_or_default();

    let hook_enabled_map = repositories::get_hook_overrides_enabled_map(&pool)
        .await
        .unwrap_or_default();

    let roles = user_ctx.roles.clone();
    let mut plugins = repositories::list_plugins_for_roles_full(
        &services_path,
        &roles,
        &enabled_map,
        &hook_enabled_map,
    )
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list plugins for roles");
        vec![]
    });

    let user_skills = repositories::list_user_skills(&pool, "admin")
        .await
        .unwrap_or_default();
    let user_agents = repositories::list_user_agents(&pool, "admin")
        .await
        .unwrap_or_default();
    if !user_skills.is_empty() || !user_agents.is_empty() {
        let skill_infos: Vec<crate::admin::types::SkillInfo> = user_skills
            .into_iter()
            .map(|s| crate::admin::types::SkillInfo {
                id: s.skill_id.clone(),
                name: s.name,
                description: s.description,
                command: format!("/custom:{}", s.skill_id),
                source: "custom".to_string(),
                enabled: s.enabled,
            })
            .collect();
        let agent_infos: Vec<crate::admin::types::AgentInfo> = user_agents
            .into_iter()
            .map(|a| crate::admin::types::AgentInfo {
                id: a.agent_id.clone(),
                name: a.name,
                description: a.description,
                enabled: a.enabled,
            })
            .collect();
        plugins.push(PluginOverview {
            id: "custom".to_string(),
            name: "Custom Skills & Agents".to_string(),
            description: "User-created custom skills and agents".to_string(),
            enabled: true,
            skills: skill_infos,
            agents: agent_infos,
            mcp_servers: vec![],
            hooks: vec![],
            depends: vec![],
        });
    }

    let data = build_config_page_data(&plugins, &services_path, &pool).await;
    super::render_page(&engine, "plugins", &data, &user_ctx, &mkt_ctx)
}

async fn build_config_page_data(
    plugins: &[PluginOverview],
    services_path: &std::path::Path,
    pool: &Arc<PgPool>,
) -> serde_json::Value {
    let all_rules = repositories::access_control::list_all_rules(pool)
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
        if rule.entity_type == "plugin" {
            rules_map
                .entry((rule.entity_type.clone(), rule.entity_id.clone()))
                .or_default()
                .push(rule);
        }
    }

    let marketplace_map = repositories::org_marketplaces::list_marketplaces_for_plugins(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch marketplace-plugin associations");
            std::collections::HashMap::new()
        });

    let known_roles = vec!["admin", "developer", "analyst", "viewer"];
    let mut categories_set = std::collections::HashSet::new();

    let plugins_json: Vec<serde_json::Value> = plugins
        .iter()
        .map(|p| {
            let detail: Option<PluginDetail> = (p.id != "custom")
                .then(|| {
                    repositories::get_plugin_detail(services_path, &p.id)
                        .ok()
                        .flatten()
                })
                .flatten();

            let version = detail.as_ref().map_or("", |d| d.version.as_str());
            let category = detail.as_ref().map_or("", |d| d.category.as_str());
            let author_name = detail.as_ref().map_or("", |d| d.author_name.as_str());
            let keywords: &[String] = detail.as_ref().map_or(&[], |d| d.keywords.as_slice());
            let depends: &[String] = &p.depends;

            if !category.is_empty() {
                categories_set.insert(category.to_string());
            }

            let yaml_roles: Vec<String> =
                detail.as_ref().map_or_else(Vec::new, |d| d.roles.clone());
            let key = ("plugin".to_string(), p.id.clone());
            let entity_rules = rules_map.get(&key);

            let mut role_names: Vec<String> = Vec::new();
            for role_name in &known_roles {
                let from_yaml = yaml_roles.iter().any(|r| r == role_name);
                let from_db = entity_rules.is_some_and(|rules| {
                    rules.iter().any(|r| {
                        r.rule_type == "role" && r.rule_value == *role_name && r.access == "allow"
                    })
                });
                if from_yaml || from_db {
                    role_names.push((*role_name).to_string());
                }
            }

            let marketplace_badges: Vec<serde_json::Value> =
                marketplace_map.get(&p.id).map_or_else(Vec::new, |mkts| {
                    mkts.iter()
                        .map(|(mkt_id, mkt_name)| json!({ "id": mkt_id, "name": mkt_name }))
                        .collect()
                });

            let mut v = serde_json::to_value(p).unwrap_or(json!({}));
            if let Some(obj) = v.as_object_mut() {
                obj.insert("skill_count".to_string(), json!(p.skills.len()));
                obj.insert("agent_count".to_string(), json!(p.agents.len()));
                obj.insert("mcp_count".to_string(), json!(p.mcp_servers.len()));
                obj.insert("hook_count".to_string(), json!(p.hooks.len()));
                obj.insert("version".to_string(), json!(version));
                obj.insert("category".to_string(), json!(category));
                obj.insert("author_name".to_string(), json!(author_name));
                obj.insert("keywords".to_string(), json!(keywords));
                obj.insert("depends_list".to_string(), json!(depends));
                obj.insert("role_names".to_string(), json!(role_names));
                obj.insert("marketplace_badges".to_string(), json!(marketplace_badges));
            }
            v
        })
        .collect();

    let plugin_count = plugins.len();
    let plugin_enabled = plugins.iter().filter(|p| p.enabled).count();

    let mut categories: Vec<String> = categories_set.into_iter().collect();
    categories.sort();

    json!({
        "page": "plugins",
        "title": "Plugins",
        "plugins": plugins_json,
        "categories": categories,
        "stats": {
            "plugin_count": plugin_count,
            "plugin_enabled": plugin_enabled,
        },
    })
}
