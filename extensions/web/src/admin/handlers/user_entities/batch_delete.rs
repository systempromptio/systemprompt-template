use std::sync::Arc;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::UserContext;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};

use super::plugins::is_entity_in_platform_plugin;

#[derive(Debug, Deserialize)]
pub(crate) struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SecretKeyItem {
    pub plugin_id: String,
    pub var_name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BatchDeleteSecretsRequest {
    pub items: Vec<SecretKeyItem>,
}

#[derive(Debug, Serialize)]
struct BatchDeleteResponse {
    deleted: usize,
    skipped: usize,
}

fn validate_ids(ids: &[String]) -> Option<Response> {
    if ids.is_empty() || ids.len() > 100 {
        return Some(shared::error_response(
            StatusCode::BAD_REQUEST,
            "ids must contain between 1 and 100 items",
        ));
    }
    None
}

fn spawn_delete_activity(pool: &Arc<PgPool>, user_id: &UserId, entity: ActivityEntity, id: &str) {
    let pool = pool.clone();
    let uid = user_id.clone();
    let entity_id = id.to_string();
    tokio::spawn(async move {
        activity::record(
            &pool,
            NewActivity::entity_deleted(&uid, entity, &entity_id, &entity_id),
        )
        .await;
    });
}

async fn finish_batch(
    pool: &Arc<PgPool>,
    user_ctx: &UserContext,
    deleted: usize,
    skipped: usize,
) -> Response {
    if deleted > 0 {
        if let Err(e) = repositories::mark_user_dirty(pool, &user_ctx.user_id).await {
            tracing::warn!(error = %e, "Failed to mark user dirty");
        }
    }
    Json(BatchDeleteResponse { deleted, skipped }).into_response()
}

pub(crate) async fn batch_delete_skills_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<BatchDeleteRequest>,
) -> Response {
    if let Some(r) = validate_ids(&req.ids) {
        return r;
    }
    let mut deleted = 0usize;
    let mut skipped = 0usize;

    for id in &req.ids {
        if is_entity_in_platform_plugin(&pool, &user_ctx.user_id, id, "skill").await {
            skipped += 1;
            continue;
        }
        let skill_id = SkillId::from(id.clone());
        match repositories::delete_user_skill(&pool, &user_ctx.user_id, &skill_id).await {
            Ok(true) => {
                spawn_delete_activity(&pool, &user_ctx.user_id, ActivityEntity::UserSkill, id);
                deleted += 1;
            }
            Ok(false) => skipped += 1,
            Err(e) => {
                tracing::warn!(error = %e, skill_id = %id, "Failed to delete skill in batch");
                skipped += 1;
            }
        }
    }
    finish_batch(&pool, &user_ctx, deleted, skipped).await
}

pub(crate) async fn batch_delete_agents_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<BatchDeleteRequest>,
) -> Response {
    if let Some(r) = validate_ids(&req.ids) {
        return r;
    }
    let mut deleted = 0usize;
    let mut skipped = 0usize;

    for id in &req.ids {
        if is_entity_in_platform_plugin(&pool, &user_ctx.user_id, id, "agent").await {
            skipped += 1;
            continue;
        }
        let agent_id = AgentId::from(id.clone());
        match repositories::delete_user_agent(&pool, &user_ctx.user_id, &agent_id).await {
            Ok(true) => {
                spawn_delete_activity(&pool, &user_ctx.user_id, ActivityEntity::UserAgent, id);
                deleted += 1;
            }
            Ok(false) => skipped += 1,
            Err(e) => {
                tracing::warn!(error = %e, agent_id = %id, "Failed to delete agent in batch");
                skipped += 1;
            }
        }
    }
    finish_batch(&pool, &user_ctx, deleted, skipped).await
}

pub(crate) async fn batch_delete_mcp_servers_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<BatchDeleteRequest>,
) -> Response {
    if let Some(r) = validate_ids(&req.ids) {
        return r;
    }
    let mut deleted = 0usize;
    let mut skipped = 0usize;

    for id in &req.ids {
        let mcp_server_id = McpServerId::new(id.clone());
        match repositories::user_mcp_servers::delete_user_mcp_server(
            &pool,
            &user_ctx.user_id,
            &mcp_server_id,
        )
        .await
        {
            Ok(true) => {
                spawn_delete_activity(&pool, &user_ctx.user_id, ActivityEntity::McpServer, id);
                deleted += 1;
            }
            Ok(false) => skipped += 1,
            Err(e) => {
                tracing::warn!(error = %e, mcp_server_id = %id, "Failed to delete MCP server in batch");
                skipped += 1;
            }
        }
    }
    finish_batch(&pool, &user_ctx, deleted, skipped).await
}

pub(crate) async fn batch_delete_hooks_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<BatchDeleteRequest>,
) -> Response {
    if let Some(r) = validate_ids(&req.ids) {
        return r;
    }
    let mut deleted = 0usize;
    let mut skipped = 0usize;

    for id in &req.ids {
        match repositories::user_hooks::delete_user_hook(&pool, id, &user_ctx.user_id).await {
            Ok(true) => {
                spawn_delete_activity(&pool, &user_ctx.user_id, ActivityEntity::Hook, id);
                deleted += 1;
            }
            Ok(false) => skipped += 1,
            Err(e) => {
                tracing::warn!(error = %e, hook_id = %id, "Failed to delete hook in batch");
                skipped += 1;
            }
        }
    }
    finish_batch(&pool, &user_ctx, deleted, skipped).await
}

pub(crate) async fn batch_delete_secrets_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<BatchDeleteSecretsRequest>,
) -> Response {
    if req.items.is_empty() || req.items.len() > 100 {
        return shared::error_response(
            StatusCode::BAD_REQUEST,
            "items must contain between 1 and 100 entries",
        );
    }

    let mut deleted = 0usize;
    let mut skipped = 0usize;

    for item in &req.items {
        match repositories::delete_plugin_env_var(
            &pool,
            &user_ctx.user_id,
            &item.plugin_id,
            &item.var_name,
        )
        .await
        {
            Ok(true) => deleted += 1,
            Ok(false) => skipped += 1,
            Err(e) => {
                tracing::warn!(
                    error = %e, plugin_id = %item.plugin_id,
                    var_name = %item.var_name, "Failed to delete secret in batch"
                );
                skipped += 1;
            }
        }
    }
    finish_batch(&pool, &user_ctx, deleted, skipped).await
}
