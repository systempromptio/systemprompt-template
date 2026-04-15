use crate::activity::ActivityEntity;
use crate::handlers::responses::ForkPluginResponse;
use crate::handlers::shared;
use crate::repositories;
use crate::types::{ForkAgentRequest, ForkPluginRequest, ForkSkillRequest, UserContext};
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use std::sync::Arc;

use super::{
    find_forkable_plugin, get_services_path, read_skill_config, spawn_fork_activity,
    tier_limit_response,
};

pub async fn fork_org_skill_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(tier_cache): Extension<crate::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkSkillRequest>,
) -> Response {
    let limit_check = crate::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::tier_limits::LimitCheck::CreateSkill,
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
    let content = super::fork_helpers::read_skill_content(&skill_dir);
    let skill_id = req.skill_id.unwrap_or_else(|| req.org_skill_id.clone());

    let create_req = crate::types::CreateSkillRequest {
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
    Extension(tier_cache): Extension<crate::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkAgentRequest>,
) -> Response {
    let limit_check = crate::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::tier_limits::LimitCheck::CreateAgent,
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
        super::super::read_agent_from_fs(&agents_path, req.org_agent_id.as_str());

    if name.is_empty() {
        return shared::error_response(StatusCode::NOT_FOUND, "Org agent not found");
    }

    let agent_id = req.agent_id.unwrap_or_else(|| req.org_agent_id.clone());
    let create_req = crate::types::CreateUserAgentRequest {
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
    Extension(tier_cache): Extension<crate::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<ForkPluginRequest>,
) -> Response {
    let limit_check = crate::tier_enforcement::check_limit(
        &tier_cache,
        pool.as_ref(),
        &user_ctx.user_id,
        crate::tier_limits::LimitCheck::CreatePlugin,
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
        Err(r) => return *r,
    };

    let result = match super::fork_helpers::fork_single_plugin(
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

async fn post_fork_cleanup(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
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
