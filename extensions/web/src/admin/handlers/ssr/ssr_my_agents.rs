use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn my_agents_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let agents = repositories::list_user_agents(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user agents");
            vec![]
        });

    let agent_ids: Vec<String> = agents.iter().map(|a| a.agent_id.clone()).collect();
    let usage_counts = repositories::fetch_agent_usage_counts(&pool, &agent_ids).await;

    let plugin_assignments: std::collections::HashMap<String, Vec<String>> =
        match sqlx::query_as::<_, (String, String)>(
            r"SELECT upa.agent_id, up.name
              FROM user_plugin_agents upa
              JOIN user_plugins up ON up.id = upa.plugin_id
              WHERE up.user_id = $1",
        )
        .bind(&user_ctx.user_id)
        .fetch_all(pool.as_ref())
        .await
        {
            Ok(rows) => {
                let mut map: std::collections::HashMap<String, Vec<String>> =
                    std::collections::HashMap::new();
                for (agent_id, plugin_name) in rows {
                    map.entry(agent_id).or_default().push(plugin_name);
                }
                map
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to fetch plugin assignments for agents");
                std::collections::HashMap::new()
            }
        };

    let agent_count = agents.len();
    let active_count = agents.iter().filter(|a| a.enabled).count();

    let agents_json: Vec<serde_json::Value> = agents
        .iter()
        .map(|a| {
            let usage = usage_counts.get(&a.agent_id).copied().unwrap_or(0);
            let prompt_preview = if a.system_prompt.len() > 200 {
                format!("{}...", &a.system_prompt[..200])
            } else {
                a.system_prompt.clone()
            };
            let is_forked = a.base_agent_id.is_some();
            let plugins = plugin_assignments
                .get(&a.agent_id)
                .cloned()
                .unwrap_or_default();
            let mut v = serde_json::to_value(a).unwrap_or(json!({}));
            if let Some(obj) = v.as_object_mut() {
                obj.insert("usage_count".to_string(), json!(usage));
                obj.insert("prompt_preview".to_string(), json!(prompt_preview));
                obj.insert("is_forked".to_string(), json!(is_forked));
                obj.insert("plugin_names".to_string(), json!(plugins));
            }
            v
        })
        .collect();

    let data = json!({
        "page": "my-agents",
        "title": "My Agents",
        "agents": agents_json,
        "stats": {
            "agent_count": agent_count,
            "active_count": active_count,
        },
    });
    super::render_page(&engine, "my-agents", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn my_agent_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let agent_id = params.get("id");
    let is_edit = agent_id.is_some();

    let agent = if let Some(id) = agent_id {
        let agents = repositories::list_user_agents(&pool, &user_ctx.user_id)
            .await
            .unwrap_or_default();
        agents.into_iter().find(|a| a.agent_id == *id)
    } else {
        None
    };

    let is_forked = agent
        .as_ref()
        .and_then(|a| a.base_agent_id.as_ref())
        .is_some();

    let data = json!({
        "page": "my-agent-edit",
        "title": if is_edit { "Edit My Agent" } else { "Create My Agent" },
        "is_edit": is_edit,
        "is_forked": is_forked,
        "agent": agent,
    });
    super::render_page(&engine, "my-agent-edit", &data, &user_ctx, &mkt_ctx)
}
