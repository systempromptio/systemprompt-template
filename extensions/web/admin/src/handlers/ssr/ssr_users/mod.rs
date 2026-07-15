//! `/admin/access/users` user roster (grouped by department) and the
//! per-user detail page.

use std::sync::Arc;

use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{IdQuery, MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use super::types::{PageStatView, UserDetailPageData, UserRuntimeView, UsersPageData};

mod data;
mod view;

pub(crate) async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let users = repositories::list_users(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list users");
        vec![]
    });

    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let total_events: i64 = users.iter().map(|u| u.total_events).sum();

    let groups = data::load_user_groups(&pool, &users).await;

    let page_stats = vec![
        PageStatView {
            value: total_users as i64,
            label: "Users",
        },
        PageStatView {
            value: active_users as i64,
            label: "Active",
        },
        PageStatView {
            value: total_events,
            label: "Events",
        },
    ];

    let data = UsersPageData {
        page: "users",
        title: "Users",
        groups,
        total_users,
        active_users,
        total_events,
        page_stats,
    };

    super::render_typed_page(&engine, "users", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn user_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdQuery>,
) -> Response {
    if !user_ctx.is_admin && Some(user_ctx.user_id.as_str()) != params.id() {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(
                "<h1>Access Denied</h1><p>You can only view your own profile.</p>",
            ),
        )
            .into_response();
    }

    let Some(id) = params.id() else {
        let data = UserDetailPageData {
            page: "user-detail",
            title: "User Detail",
            user: None,
            gamification: None,
            not_found: true,
            user_department: String::new(),
            user_assignments: super::types::UserAssignmentSummary::default(),
            user_devices: Vec::new(),
            user_devices_count: 0,
            departments: Vec::new(),
            runtime: None,
            effective_permissions: None,
            has_effective_permissions: false,
        };
        return super::render_typed_page(&engine, "user-detail", &data, &user_ctx, &mkt_ctx);
    };
    let user_id = UserId::new(id);

    let detail = repositories::find_user_detail(&pool, &user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user detail");
        })
        .ok()
        .flatten();
    let gamification: Option<crate::types::UserGamificationProfile> = None;

    let not_found = detail.is_none();

    let (user_department, user_assignments, user_devices, user_devices_count, effective) =
        match detail.as_ref() {
            Some(d) => data::collect_user_detail_extras(&pool, d).await,
            None => (
                String::new(),
                super::types::UserAssignmentSummary::default(),
                Vec::new(),
                0,
                None,
            ),
        };

    let runtime = match detail.as_ref() {
        Some(d) => load_runtime_view(&pool, d).await,
        None => None,
    };

    let departments = data::fetch_departments(&pool).await;

    let has_effective_permissions = effective
        .as_ref()
        .is_some_and(|eff| !eff.gateway_routes.is_empty() || !eff.mcp_servers.is_empty());

    let data = UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: detail,
        gamification,
        not_found,
        user_department,
        user_assignments,
        user_devices,
        user_devices_count,
        departments,
        runtime,
        effective_permissions: effective,
        has_effective_permissions,
    };
    super::render_typed_page(&engine, "user-detail", &data, &user_ctx, &mkt_ctx)
}

async fn load_runtime_view(pool: &PgPool, d: &crate::types::UserDetail) -> Option<UserRuntimeView> {
    repositories::get_user_runtime_detail(pool, &d.user_id)
        .await
        .ok()
        .map(|r| UserRuntimeView {
            connected_agents: r.connected_agents,
            total_agents: r.total_agents,
            tokens_in: r.tokens_in,
            tokens_out: r.tokens_out,
            last_bridge_version: r.last_bridge_version,
            last_os: r.last_os,
            last_hostname: r.last_hostname,
            last_heartbeat_at: r.last_heartbeat_at.map(|t| t.to_rfc3339()),
        })
}
