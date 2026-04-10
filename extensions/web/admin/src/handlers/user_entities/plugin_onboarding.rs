use std::sync::Arc;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::repositories;
use crate::types::UserContext;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use super::fork::fork_single_plugin;
use super::plugin_selections::SelectPluginsRequest;
use crate::handlers::shared;

const MAX_ONBOARDING_PLUGINS: usize = 3;

#[derive(Serialize)]
struct SelectAndForkResponse {
    redirect_url: &'static str,
    forked_count: usize,
    skipped_count: usize,
}

pub async fn select_and_fork_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<SelectPluginsRequest>,
) -> Response {
    if let Some(resp) = validate_plugin_selection(&req) {
        return resp;
    }

    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let valid_ids = match resolve_valid_plugin_ids(&pool, req.plugin_ids).await {
        Ok(ids) => ids,
        Err(resp) => return resp,
    };

    if let Err(e) = repositories::user_plugin_selections::set_selected_org_plugins(
        &pool,
        &user_ctx.user_id,
        &valid_ids,
    )
    .await
    {
        tracing::error!(error = %e, "Failed to set selected plugins during onboarding");
        return shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save plugin selections",
        );
    }

    let (forked_count, skipped_count) =
        fork_valid_plugins(&pool, &user_ctx, &valid_ids, &services_path).await;

    spawn_onboarding_activity(&pool, &user_ctx, forked_count);

    Json(SelectAndForkResponse {
        redirect_url: "/admin/my/marketplace/",
        forked_count,
        skipped_count,
    })
    .into_response()
}

fn validate_plugin_selection(req: &SelectPluginsRequest) -> Option<Response> {
    if req.plugin_ids.len() > MAX_ONBOARDING_PLUGINS {
        return Some(shared::error_response(
            StatusCode::BAD_REQUEST,
            "You can select a maximum of 3 plugins",
        ));
    }
    if req.plugin_ids.is_empty() {
        return Some(shared::error_response(
            StatusCode::BAD_REQUEST,
            "At least one plugin is required",
        ));
    }
    None
}

async fn resolve_valid_plugin_ids(
    pool: &PgPool,
    plugin_ids: Vec<String>,
) -> Result<Vec<String>, Response> {
    let authorized = repositories::org_marketplaces::resolve_authorized_org_plugin_ids(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to resolve authorized org plugin IDs");
            std::collections::HashSet::new()
        });

    let valid_ids: Vec<String> = plugin_ids
        .into_iter()
        .filter(|id| authorized.contains(id))
        .collect();

    if valid_ids.is_empty() {
        return Err(shared::error_response(
            StatusCode::BAD_REQUEST,
            "No valid plugins in the selection",
        ));
    }

    Ok(valid_ids)
}

fn spawn_onboarding_activity(pool: &Arc<PgPool>, user_ctx: &UserContext, forked_count: usize) {
    if forked_count > 0 {
        let pool = Arc::clone(pool);
        let uid = user_ctx.user_id.clone();
        let desc = format!("Selected and forked {forked_count} plugin(s) during onboarding");
        tokio::spawn(async move {
            activity::record(
                &pool,
                NewActivity::entity_created(&uid, ActivityEntity::Plugin, "onboarding", &desc),
            )
            .await;
        });
    }
}

async fn fork_valid_plugins(
    pool: &PgPool,
    user_ctx: &UserContext,
    valid_ids: &[String],
    services_path: &std::path::Path,
) -> (usize, usize) {
    let org_plugins = repositories::list_plugins_for_roles(services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for onboarding fork");
            Vec::new()
        });

    let existing_base_ids = collect_existing_base_ids(pool, &user_ctx.user_id).await;

    let mut forked_count = 0;
    let mut skipped_count = 0;

    for plugin_id in valid_ids {
        if existing_base_ids.contains(plugin_id) {
            skipped_count += 1;
            continue;
        }

        let Some(org_plugin) = org_plugins.iter().find(|p| &p.id == plugin_id) else {
            tracing::warn!(plugin_id = %plugin_id, "Org plugin not found during onboarding fork");
            continue;
        };

        if try_fork_plugin(pool, user_ctx, org_plugin, services_path, plugin_id).await {
            forked_count += 1;
        }
    }

    if let Err(e) = repositories::mark_user_dirty(pool, &user_ctx.user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty after onboarding fork");
    }

    (forked_count, skipped_count)
}

async fn collect_existing_base_ids(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> std::collections::HashSet<String> {
    repositories::list_user_plugins(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = ?e, "Failed to list user plugins for onboarding");
            Vec::new()
        })
        .iter()
        .filter_map(|p| p.base_plugin_id.clone())
        .collect()
}

async fn try_fork_plugin(
    pool: &PgPool,
    user_ctx: &UserContext,
    org_plugin: &crate::types::PluginOverview,
    services_path: &std::path::Path,
    plugin_id: &str,
) -> bool {
    match fork_single_plugin(
        pool,
        &user_ctx.user_id,
        &user_ctx.username,
        org_plugin,
        services_path,
        None,
    )
    .await
    {
        Ok(_result) => {
            if let Err(e) = repositories::user_plugin_selections::remove_selected_org_plugin(
                pool,
                &user_ctx.user_id,
                plugin_id,
            )
            .await
            {
                tracing::warn!(error = %e, plugin_id = %plugin_id, "Failed to remove forked org plugin from selections");
            }
            true
        }
        Err(msg) => {
            tracing::error!(error = %msg, plugin_id = %plugin_id, "Failed to fork plugin during onboarding");
            false
        }
    }
}
