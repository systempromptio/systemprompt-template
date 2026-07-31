//! `/admin/access/users` user roster (grouped by department) and the
//! per-user detail page.

use std::sync::Arc;

use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{IdQuery, MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use super::types::{PageStatView, UserDetailPageData, UserRuntimeView, UsersPageData};

mod data;
mod view;

pub(crate) async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let users = repositories::users::queries::list_users(&pool)
        .await
        .unwrap_or_else(|e| {
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

    Ok(super::render_typed_page(
        &engine, "users", &data, &user_ctx, &mkt_ctx,
    ))
}

pub(crate) async fn user_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin && Some(user_ctx.user_id.as_str()) != params.id() {
        return Err(AdminError::Forbidden("You can only view your own profile.".to_owned()).into());
    }

    let Some(id) = params.id() else {
        let data = blank_user_detail();
        return Ok(super::render_typed_page(
            &engine,
            "user-detail",
            &data,
            &user_ctx,
            &mkt_ctx,
        ));
    };
    let user_id = UserId::new(id);

    // Why: `Err` and `Ok(None)` must not collapse together here: this value alone
    // decides whether the page renders "User not found.", so a failed query
    // would tell an admin the account had been deleted. Only a genuine absence
    // is a not-found.
    let detail = repositories::users::queries::find_user_detail(&pool, &user_id).await?;
    let gamification: Option<crate::types::UserGamificationProfile> = None;

    let not_found = detail.is_none();

    let (user_department, user_assignments, user_tokens, user_tokens_count, effective) =
        match detail.as_ref() {
            Some(d) => data::collect_user_detail_extras(&pool, d).await?,
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

    // Why: the department `<select>` marks an option selected by matching this
    // value, so an empty one would leave nothing selected and let the browser
    // pick the first department in the list.
    let user_department = if user_department.is_empty() && !not_found {
        crate::types::departments::DEFAULT_DEPARTMENT.to_owned()
    } else {
        user_department
    };
    let departments = data::fetch_departments(&pool, &user_department).await;

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
        user_tokens,
        user_tokens_count,
        departments,
        runtime,
        effective_permissions: effective,
        has_effective_permissions,
    };
    Ok(super::render_typed_page(
        &engine,
        "user-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn blank_user_detail() -> UserDetailPageData {
    UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: None,
        gamification: None,
        not_found: true,
        user_department: String::new(),
        user_assignments: super::types::UserAssignmentSummary::default(),
        user_tokens: Vec::new(),
        user_tokens_count: 0,
        departments: Vec::new(),
        runtime: None,
        effective_permissions: None,
        has_effective_permissions: false,
    }
}

async fn load_runtime_view(pool: &PgPool, d: &crate::types::UserDetail) -> Option<UserRuntimeView> {
    repositories::users::queries::get_user_runtime_detail(pool, &d.user_id)
        .await
        .ok()
        .map(|r| UserRuntimeView {
            requests: r.requests,
            tokens_in: r.tokens_in,
            tokens_out: r.tokens_out,
            last_request_at: r.last_request_at.map(|t| t.to_rfc3339()),
        })
}
