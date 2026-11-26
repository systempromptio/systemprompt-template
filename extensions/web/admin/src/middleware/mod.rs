//! Admin plane request middleware: session resolution and page context.
//!
//! `user_context_middleware` runs first and puts a [`UserContext`] on the
//! request only for a signed-in principal (see [`crate::types::NotSignedIn`]);
//! [`gates`] then decides whether the request may proceed, and
//! `marketplace_context_middleware` supplies what a page needs to render.
//!
//! The marketplace counts injected into every render are cached because they
//! are derived from a remote catalog and are identical for every user holding
//! the same role set.

use crate::env::AdminEnv;

mod gates;
mod pat;

pub(crate) use gates::{
    non_admin_gate_middleware, require_auth_middleware, require_roles_middleware,
    require_user_middleware,
};

use std::sync::Arc;

use axum::Extension;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use super::handlers::extract_user_from_cookie;
use super::types::UserContext;

#[derive(Debug, Serialize)]
struct AuthMeResponse {
    user_id: UserId,
    username: String,
    email: Email,
    roles: Vec<String>,
    group_ids: Vec<String>,
    project_ids: Vec<String>,
    is_admin: bool,
    is_console: bool,
    is_platform_admin: bool,
}

pub(crate) use super::marketplace_context::marketplace_context_middleware;

pub(crate) async fn user_context_middleware(
    State(pool): State<Arc<PgPool>>,
    Extension(env): Extension<AdminEnv>,
    mut request: Request,
    next: Next,
) -> Response {
    let cookie = extract_user_from_cookie(&env, request.headers()).await;
    let pat = cookie
        .is_err()
        .then(|| pat::pat_request(&request))
        .flatten();
    let session = match cookie {
        Ok(s) => s,
        Err(reason) => match pat::pat_session(&pool, pat).await {
            pat::PatSession::Accepted(s) => s,
            pat::PatSession::Rejected(response) => return response,
            pat::PatSession::NotApplicable => {
                tracing::warn!(reason = %reason, "UserContext middleware: no valid session");
                return next.run(request).await;
            },
        },
    };

    let account = find_access_profile(&pool, &session.user_id).await;
    let user_id = session.user_id.clone();
    match UserContext::signed_in(session, account) {
        Ok(ctx) => {
            request.extensions_mut().insert(ctx);
        },
        Err(refusal) => {
            tracing::info!(user_id = %user_id, reason = %refusal, "UserContext middleware: not a signed-in principal");
        },
    }
    next.run(request).await
}

async fn find_access_profile(
    pool: &PgPool,
    user_id: &UserId,
) -> Option<super::repositories::users::queries::UserAccessProfile> {
    super::repositories::users::queries::find_user_access_profile(pool, user_id)
        .await
        .inspect_err(
            |e| tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user roles"),
        )
        .ok()
        .flatten()
}

pub(crate) async fn auth_me_handler(Extension(user_ctx): Extension<UserContext>) -> Response {
    if user_ctx.user_id.as_str().is_empty() {
        return (StatusCode::UNAUTHORIZED, "Not authenticated").into_response();
    }
    axum::Json(AuthMeResponse {
        user_id: user_ctx.user_id,
        username: user_ctx.username,
        email: user_ctx.email,
        roles: user_ctx.roles,
        group_ids: user_ctx.group_ids,
        project_ids: user_ctx.project_ids,
        is_admin: user_ctx.is_admin,
        is_console: user_ctx.is_console,
        is_platform_admin: user_ctx.is_platform_admin,
    })
    .into_response()
}
