use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::{UpsertSkillSecretRequest, UserContext};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;

use systemprompt::identifiers::SkillId;

use crate::admin::handlers::{responses::SecretsListResponse, shared};

#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    pub plugin_id: String,
    pub var_name: String,
    pub var_value: String,
    #[serde(default)]
    pub is_secret: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSecretRequest {
    pub var_value: String,
    #[serde(default)]
    pub is_secret: bool,
}

pub async fn list_user_secrets_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::list_all_user_env_vars(&pool, &user_ctx.user_id).await {
        Ok(vars) => Json(SecretsListResponse { secrets: vars }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list user secrets");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list secrets")
        }
    }
}

pub async fn create_user_secret_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateSecretRequest>,
) -> Response {
    match repositories::upsert_plugin_env_var(
        &pool,
        &user_ctx.user_id,
        &req.plugin_id,
        &req.var_name,
        &req.var_value,
        req.is_secret,
    )
    .await
    {
        Ok(()) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            StatusCode::CREATED.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create secret");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create secret")
        }
    }
}

pub async fn update_user_secret_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path((plugin_id, var_name)): Path<(String, String)>,
    Json(req): Json<UpdateSecretRequest>,
) -> Response {
    match repositories::upsert_plugin_env_var(
        &pool,
        &user_ctx.user_id,
        &plugin_id,
        &var_name,
        &req.var_value,
        req.is_secret,
    )
    .await
    {
        Ok(()) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to update secret");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update secret")
        }
    }
}

pub async fn delete_user_secret_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path((plugin_id, var_name)): Path<(String, String)>,
) -> Response {
    match repositories::delete_plugin_env_var(&pool, &user_ctx.user_id, &plugin_id, &var_name).await
    {
        Ok(true) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Secret not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete secret");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete secret")
        }
    }
}

pub async fn list_skill_secrets_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
) -> Response {
    let skill_id = SkillId::from(skill_id);
    match repositories::list_skill_secrets(&pool, &user_ctx.user_id, &skill_id).await {
        Ok(secrets) => Json(SecretsListResponse { secrets }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list skill secrets");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list skill secrets",
            )
        }
    }
}

pub async fn upsert_skill_secret_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
    Json(req): Json<UpsertSkillSecretRequest>,
) -> Response {
    let skill_id = SkillId::from(skill_id);
    match repositories::upsert_skill_secret(
        &pool,
        &user_ctx.user_id,
        &skill_id,
        &req.var_name,
        &req.var_value,
    )
    .await
    {
        Ok(()) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to upsert skill secret");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save skill secret",
            )
        }
    }
}

pub async fn delete_skill_secret_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path((skill_id, var_name)): Path<(String, String)>,
) -> Response {
    let skill_id = SkillId::from(skill_id);
    match repositories::delete_skill_secret(&pool, &user_ctx.user_id, &skill_id, &var_name).await {
        Ok(true) => {
            if let Err(e) = repositories::mark_user_dirty(&pool, &user_ctx.user_id).await {
                tracing::warn!(error = %e, "Failed to mark user dirty");
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Skill secret not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete skill secret");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete skill secret",
            )
        }
    }
}
