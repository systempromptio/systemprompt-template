mod handlers;

use super::fork_helpers;
pub use super::fork_helpers::fork_single_plugin;
use crate::activity::{self, ActivityEntity, NewActivity};
use crate::handlers::shared;
use crate::repositories;
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use systemprompt::identifiers::UserId;

pub use handlers::{fork_org_agent_handler, fork_org_plugin_handler, fork_org_skill_handler};

fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    shared::get_services_path()
}

fn tier_limit_response(
    entity_type: &str,
    limit_check: &crate::tier_limits::LimitCheckResult,
) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": "entity_limit_reached",
            "entity_type": entity_type,
            "message": limit_check.reason,
            "limit": limit_check.limit_value,
            "current": limit_check.current_value,
        })),
    )
        .into_response()
}

fn spawn_fork_activity(
    pool: &sqlx::PgPool,
    user_id: &UserId,
    entity: ActivityEntity,
    id: &str,
    name: &str,
) {
    let pool = pool.clone();
    let uid = user_id.clone();
    let id = id.to_string();
    let name = name.to_string();
    tokio::spawn(async move {
        activity::record(
            &pool,
            NewActivity::entity_forked(uid.as_str(), entity, &id, &name),
        )
        .await;
    });
}

fn read_skill_config(
    skill_dir: &std::path::Path,
    org_skill_id: &str,
) -> (String, String, Vec<String>) {
    let config_path = skill_dir.join("config.yaml");
    if !config_path.exists() {
        return (org_skill_id.to_string(), String::new(), vec![]);
    }
    let cfg_text = std::fs::read_to_string(&config_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, path = %config_path.display(), "Failed to read skill config for fork");
        String::new()
    });
    let cfg: serde_yaml::Value = serde_yaml::from_str(&cfg_text).unwrap_or_else(|e| {
        tracing::warn!(error = %e, path = %config_path.display(), "Failed to parse skill config YAML for fork");
        serde_yaml::Value::Null
    });
    let name = cfg
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(org_skill_id)
        .to_string();
    let desc = cfg
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tags: Vec<String> =
        cfg.get("tags")
            .and_then(|v| v.as_sequence())
            .map_or_else(Vec::new, |seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });
    (name, desc, tags)
}

fn find_forkable_plugin(
    services_path: &std::path::Path,
    roles: &[String],
    org_plugin_id: &str,
) -> Result<crate::types::PluginOverview, Box<Response>> {
    if org_plugin_id == "systemprompt" {
        return Err(Box::new(shared::error_response(
            StatusCode::FORBIDDEN,
            "Platform plugin cannot be forked",
        )));
    }
    let org_plugins =
        repositories::list_plugins_for_roles(services_path, roles).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for fork");
            Vec::new()
        });
    org_plugins
        .into_iter()
        .find(|p| p.id == org_plugin_id)
        .ok_or_else(|| {
            Box::new(shared::error_response(
                StatusCode::NOT_FOUND,
                "Org plugin not found or not accessible",
            ))
        })
}
