pub use super::fork_helpers::fork_single_plugin;
use super::fork_helpers::read_skill_content;
use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::handlers::responses::ForkPluginResponse;
use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::{ForkAgentRequest, ForkPluginRequest, ForkSkillRequest, UserContext};
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;

fn get_services_path() -> Result<std::path::PathBuf, Box<Response>> {
    shared::get_services_path()
}

fn tier_limit_response(
    entity_type: &str,
    limit_check: &crate::admin::tier_limits::LimitCheckResult,
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
    pool: &PgPool,
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

pub async fn fork_org_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::admin::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkSkillRequest>,
) -> Response {
    let limit_check = crate::admin::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::admin::tier_limits::LimitCheck::CreateSkill,
    )
    .await;
    if !limit_check.allowed {
        return tier_limit_response("skill", &limit_check);
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let skill_dir = services_path.join("skills").join(req.org_skill_id.as_str());
    if !skill_dir.exists() {
        return shared::error_response(StatusCode::NOT_FOUND, "Org skill not found");
    }

    let (name, description, tags) = read_skill_config(&skill_dir, req.org_skill_id.as_str());
    let content = read_skill_content(&skill_dir);
    let skill_id = req.skill_id.unwrap_or_else(|| req.org_skill_id.clone());

    let create_req = crate::admin::types::CreateSkillRequest {
        skill_id: skill_id.clone(),
        name,
        description,
        content,
        tags,
        base_skill_id: Some(req.org_skill_id),
    };

    match repositories::create_user_skill(&pool, &user_ctx.user_id, &create_req).await {
        Ok(skill) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            spawn_fork_activity(
                &pool,
                &user_ctx.user_id,
                ActivityEntity::UserSkill,
                &skill.id,
                &skill.name,
            );
            (StatusCode::CREATED, Json(skill)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fork org skill");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fork skill")
        }
    }
}

pub async fn fork_org_agent_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::admin::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkAgentRequest>,
) -> Response {
    let limit_check = crate::admin::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::admin::tier_limits::LimitCheck::CreateAgent,
    )
    .await;
    if !limit_check.allowed {
        return tier_limit_response("agent", &limit_check);
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let agents_path = services_path.join("agents");
    let (name, description, system_prompt) =
        super::read_agent_from_fs(&agents_path, req.org_agent_id.as_str());

    if name.is_empty() {
        return shared::error_response(StatusCode::NOT_FOUND, "Org agent not found");
    }

    let agent_id = req.agent_id.unwrap_or_else(|| req.org_agent_id.clone());
    let create_req = crate::admin::types::CreateUserAgentRequest {
        agent_id: agent_id.clone(),
        name,
        description,
        system_prompt,
        base_agent_id: Some(req.org_agent_id),
    };

    match repositories::create_user_agent(&pool, &user_ctx.user_id, &create_req).await {
        Ok(agent) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            spawn_fork_activity(
                &pool,
                &user_ctx.user_id,
                ActivityEntity::UserAgent,
                &agent.id,
                &agent.name,
            );
            (StatusCode::CREATED, Json(agent)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fork org agent");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fork agent")
        }
    }
}

pub async fn fork_org_plugin_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::admin::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkPluginRequest>,
) -> Response {
    let limit_check = crate::admin::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::admin::tier_limits::LimitCheck::CreatePlugin,
    )
    .await;
    if !limit_check.allowed {
        return tier_limit_response("plugin", &limit_check);
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let org_plugin = match find_forkable_plugin(&services_path, &user_ctx.roles, &req.org_plugin_id)
    {
        Ok(p) => p,
        Err(r) => return r,
    };

    let result = match fork_single_plugin(
        &pool,
        &user_ctx.user_id,
        &user_ctx.username,
        &org_plugin,
        &services_path,
        req.plugin_id,
    )
    .await
    {
        Ok(r) => r,
        Err(msg) => {
            tracing::error!(error = %msg, "Failed to fork plugin");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fork plugin",
            );
        }
    };

    post_fork_cleanup(&pool, &user_ctx.user_id, &req.org_plugin_id, &result).await;

    (
        StatusCode::CREATED,
        Json(ForkPluginResponse {
            plugin: result.plugin,
            forked_skills: result.forked_skills,
            forked_agents: result.forked_agents,
        }),
    )
        .into_response()
}

fn find_forkable_plugin(
    services_path: &std::path::Path,
    roles: &[String],
    org_plugin_id: &str,
) -> Result<crate::admin::types::PluginOverview, Response> {
    if org_plugin_id == "systemprompt" {
        return Err(shared::error_response(
            StatusCode::FORBIDDEN,
            "Platform plugin cannot be forked",
        ));
    }
    let org_plugins = repositories::list_plugins_for_roles(services_path, roles).unwrap_or_else(
        |e| {
            tracing::warn!(error = %e, "Failed to list plugins for fork");
            Vec::new()
        },
    );
    org_plugins
        .into_iter()
        .find(|p| p.id == org_plugin_id)
        .ok_or_else(|| {
            shared::error_response(
                StatusCode::NOT_FOUND,
                "Org plugin not found or not accessible",
            )
        })
}

async fn post_fork_cleanup(
    pool: &PgPool,
    user_id: &UserId,
    org_plugin_id: &str,
    result: &super::fork_helpers::ForkSinglePluginResult,
) {
    if let Err(e) = repositories::mark_user_dirty(pool, user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty");
    }
    if let Err(e) = repositories::user_plugin_selections::remove_selected_org_plugin(
        pool,
        user_id,
        org_plugin_id,
    )
    .await
    {
        tracing::warn!(error = %e, org_plugin_id = %org_plugin_id, "Failed to remove forked org plugin from selections");
    }
    spawn_fork_activity(
        pool,
        user_id,
        ActivityEntity::Plugin,
        &result.plugin.id,
        &result.plugin.name,
    );
}
