use std::sync::Arc;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::repositories;
use crate::admin::types::UserContext;
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use super::fork::fork_single_plugin;
use super::plugin_selections::SelectPluginsRequest;
use crate::admin::handlers::shared;

const MAX_ONBOARDING_PLUGINS: usize = 3;

#[derive(Serialize)]
struct SelectAndForkResponse {
    redirect_url: &'static str,
    forked_count: usize,
    skipped_count: usize,
}

pub(crate) async fn select_and_fork_plugins_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<SelectPluginsRequest>,
) -> Response {
    if req.plugin_ids.len() > MAX_ONBOARDING_PLUGINS {
        return shared::error_response(
            StatusCode::BAD_REQUEST,
            "You can select a maximum of 3 plugins",
        );
    }

    if req.plugin_ids.is_empty() {
        return shared::error_response(StatusCode::BAD_REQUEST, "At least one plugin is required");
    }

    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let authorized = repositories::org_marketplaces::resolve_authorized_org_plugin_ids(&pool)
        .await
        .unwrap_or_else(|_| std::collections::HashSet::new());

    let valid_ids: Vec<String> = req
        .plugin_ids
        .into_iter()
        .filter(|id| id != "systemprompt" && authorized.contains(id))
        .collect();

    if valid_ids.is_empty() {
        return shared::error_response(
            StatusCode::BAD_REQUEST,
            "No valid plugins in the selection",
        );
    }

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

    if forked_count > 0 {
        let pool = pool.clone();
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

    Json(SelectAndForkResponse {
        redirect_url: "/admin/my/marketplace/",
        forked_count,
        skipped_count,
    })
    .into_response()
}

async fn fork_valid_plugins(
    pool: &Arc<PgPool>,
    user_ctx: &UserContext,
    valid_ids: &[String],
    services_path: &std::path::Path,
) -> (usize, usize) {
    let org_plugins = repositories::list_plugins_for_roles(services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for onboarding fork");
            Vec::new()
        });

    let existing_plugins = repositories::list_user_plugins(pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|_| Vec::new());
    let existing_base_ids: std::collections::HashSet<String> = existing_plugins
        .iter()
        .filter_map(|p| p.base_plugin_id.clone())
        .collect();

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
                forked_count += 1;
                if let Err(e) = repositories::user_plugin_selections::remove_selected_org_plugin(
                    pool,
                    &user_ctx.user_id,
                    plugin_id,
                )
                .await
                {
                    tracing::warn!(error = %e, plugin_id = %plugin_id, "Failed to remove forked org plugin from selections");
                }
            }
            Err(msg) => {
                tracing::error!(error = %msg, plugin_id = %plugin_id, "Failed to fork plugin during onboarding");
            }
        }
    }

    if let Err(e) = repositories::mark_user_dirty(pool, &user_ctx.user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty after onboarding fork");
    }

    (forked_count, skipped_count)
}
