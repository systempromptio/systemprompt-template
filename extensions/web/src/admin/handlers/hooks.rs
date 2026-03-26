use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::repositories;
use crate::admin::repositories::hook_catalog;
use crate::admin::types::{CreateHookRequest, UpdateHookRequest, UserContext};

use super::resources::get_services_path;

pub(crate) async fn list_hooks_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let catalog_hooks = match hook_catalog::list_catalog_hooks(&pool).await {
        Ok(hooks) if !hooks.is_empty() => hooks,
        _ => match hook_catalog::list_file_hooks(&services_path) {
            Ok(hooks) => hooks,
            Err(e) => {
                tracing::error!(error = %e, "Failed to list hooks");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        },
    };

    let hooks: Vec<_> = catalog_hooks
        .iter()
        .map(hook_catalog::catalog_to_detail)
        .collect();

    if user_ctx.is_admin {
        return Json(hooks).into_response();
    }

    let plugins =
        repositories::list_plugins_for_roles(&services_path, &user_ctx.roles).unwrap_or_default();
    let visible_ids: std::collections::HashSet<String> =
        plugins.iter().map(|p| p.id.clone()).collect();

    let filtered: Vec<_> = hooks
        .into_iter()
        .filter(|h| {
            h.plugin_id.is_empty() // system hooks with no specific plugin
                || visible_ids.contains(&h.plugin_id)
        })
        .collect();
    Json(filtered).into_response()
}

pub(crate) async fn get_hook_handler(
    State(pool): State<Arc<PgPool>>,
    Path(hook_id): Path<String>,
) -> Response {
    match hook_catalog::get_catalog_hook(&pool, &hook_id).await {
        Ok(Some(hook)) => {
            let detail = hook_catalog::catalog_to_detail(&hook);
            return Json(detail).into_response();
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(error = %e, "Catalog lookup failed, trying file-based");
        }
    }

    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match repositories::hooks::get_hook(&services_path, &hook_id) {
        Ok(Some(hook)) => Json(hook).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Hook not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn create_hook_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateHookRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match hook_catalog::create_catalog_hook(&pool, &services_path, &body).await {
        Ok(hook) => {
            let name = hook.name.clone();
            let hid = hook.id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_created(&uid, ActivityEntity::Hook, &hid, &name),
                )
                .await;
            });
            let detail = hook_catalog::catalog_to_detail(&hook);
            (StatusCode::CREATED, Json(detail)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn update_hook_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(hook_id): Path<String>,
    Json(body): Json<UpdateHookRequest>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match hook_catalog::update_catalog_hook(&pool, &services_path, &hook_id, &body).await {
        Ok(Some(hook)) => {
            let name = hook.name.clone();
            let hid = hook_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_updated(&uid, ActivityEntity::Hook, &hid, &name),
                )
                .await;
            });
            let detail = hook_catalog::catalog_to_detail(&hook);
            Json(detail).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Hook not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn toggle_hook_handler(
    State(pool): State<Arc<PgPool>>,
    Path(hook_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let enabled = body
        .get("enabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    match repositories::upsert_hook_override_enabled(&pool, &hook_id, enabled).await {
        Ok(()) => Json(serde_json::json!({"hook_id": hook_id, "enabled": enabled})).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to toggle hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn delete_hook_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(hook_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        )
            .into_response();
    }
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    match hook_catalog::delete_catalog_hook(&pool, &services_path, &hook_id).await {
        Ok(true) => {
            let hid = hook_id.clone();
            let uid = user_ctx.user_id.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_deleted(&uid, ActivityEntity::Hook, &hid, &hid),
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Hook not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete hook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
