use std::sync::Arc;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::repositories;
use crate::admin::types::UserContext;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::admin::handlers::shared;

#[derive(Serialize)]
struct SelectedPluginsResponse {
    plugin_ids: Vec<String>,
    has_selections: bool,
}

#[derive(Serialize)]
struct AvailablePluginsResponse {
    plugins: Vec<AvailablePlugin>,
}

#[derive(Serialize)]
struct AvailablePlugin {
    plugin_id: String,
    name: String,
    description: String,
    category: String,
    skill_count: usize,
    agent_count: usize,
    mcp_count: usize,
    selected: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SelectPluginsRequest {
    pub plugin_ids: Vec<String>,
}

pub(crate) async fn list_available_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let authorized = repositories::org_marketplaces::resolve_authorized_org_plugin_ids(&pool)
        .await
        .unwrap_or_default();

    let selected =
        repositories::user_plugin_selections::list_selected_org_plugins(&pool, &user_ctx.user_id)
            .await
            .unwrap_or_default();

    let selected_set: std::collections::HashSet<&str> =
        selected.iter().map(String::as_str).collect();

    let plugins_path = services_path.join("plugins");
    let mut plugins: Vec<AvailablePlugin> = Vec::new();

    for plugin_id in &authorized {
        if plugin_id == "systemprompt" {
            continue;
        }
        let detail = repositories::find_plugin_detail(&services_path, plugin_id)
            .ok()
            .flatten();
        let (name, description, category, skill_count, agent_count, mcp_count) =
            if let Some(ref d) = detail {
                (
                    d.name.clone(),
                    d.description.clone(),
                    d.category.clone(),
                    d.skills.len(),
                    d.agents.len(),
                    d.mcp_servers.len(),
                )
            } else {
                if !plugins_path.join(plugin_id).exists() {
                    continue;
                }
                (plugin_id.clone(), String::new(), String::new(), 0, 0, 0)
            };

        plugins.push(AvailablePlugin {
            plugin_id: plugin_id.clone(),
            name,
            description,
            category,
            skill_count,
            agent_count,
            mcp_count,
            selected: selected_set.contains(plugin_id.as_str()),
        });
    }

    plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Json(AvailablePluginsResponse { plugins }).into_response()
}

pub(crate) async fn list_selected_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::user_plugin_selections::list_selected_org_plugins(&pool, &user_ctx.user_id)
        .await
    {
        Ok(plugin_ids) => {
            let has_selections = !plugin_ids.is_empty();
            Json(SelectedPluginsResponse {
                plugin_ids,
                has_selections,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to list selected plugins");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list selected plugins",
            )
        }
    }
}

pub(crate) async fn set_selected_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<SelectPluginsRequest>,
) -> Response {
    let authorized = repositories::org_marketplaces::resolve_authorized_org_plugin_ids(&pool)
        .await
        .unwrap_or_default();

    let valid_ids: Vec<String> = req
        .plugin_ids
        .into_iter()
        .filter(|id| id != "systemprompt" && authorized.contains(id))
        .collect();

    match repositories::user_plugin_selections::set_selected_org_plugins(
        &pool,
        &user_ctx.user_id,
        &valid_ids,
    )
    .await
    {
        Ok(()) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty after plugin selection");
            }
            let count = valid_ids.len();
            let pool = pool.clone();
            let uid = user_ctx.user_id.clone();
            let desc = format!("Updated plugin selections ({count} plugins selected)");
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_updated(
                        &uid,
                        ActivityEntity::Plugin,
                        "plugin-selections",
                        &desc,
                    ),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to set selected plugins");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save plugin selections",
            )
        }
    }
}
