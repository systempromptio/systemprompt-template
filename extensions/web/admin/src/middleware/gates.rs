//! Authentication and authorisation layers for the admin plane.
//!
//! These sit above `user_context_middleware`, which is what populates the
//! [`UserContext`] they read. They are separated from page context because
//! they answer a different question: context decides what a page renders,
//! these decide whether the request is allowed to reach one at all.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::handlers::shared::ErrorBody;
use crate::types::UserContext;

/// The request path as the client sent it.
///
/// `nest_service` strips its prefix from `request.uri()`, so a layer inside
/// the admin SSR router sees `/profile` where the caller asked for
/// `/admin/profile`. Anything matching against user-facing paths has to read
/// through `OriginalUri` instead.
fn original_path(request: &Request) -> String {
    request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map_or_else(
            || request.uri().path().to_owned(),
            |o| o.0.path().to_owned(),
        )
}

pub(crate) async fn require_user_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => {
            let uri = original_path(&request);
            let redirect_url = format!("/admin/login?redirect={uri}");
            axum::response::Redirect::temporary(&redirect_url).into_response()
        },
    }
}

pub(crate) async fn require_auth_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(ErrorBody {
                error: "Authentication required".to_owned(),
            }),
        )
            .into_response(),
    }
}

pub(crate) async fn require_admin_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if ctx.is_admin => next.run(request).await,
        _ => (
            StatusCode::FORBIDDEN,
            axum::Json(ErrorBody {
                error: "Admin access required".to_owned(),
            }),
        )
            .into_response(),
    }
}

/// Restrict non-admin users to the profile page, settings page, and a few
/// account-management endpoints. Other admin routes redirect to /admin/profile.
///
/// Admins pass through unchanged. Anonymous users are handled by
/// `require_user_middleware` which runs after this layer.
///
/// The path comes from `OriginalUri`, as it does in `require_user_middleware`:
/// this layer sits inside a `nest_service("/admin", …)`, which strips the
/// prefix, and every arm of `is_non_admin_allowed_path` matches on it.
pub(crate) async fn non_admin_gate_middleware(request: Request, next: Next) -> Response {
    let path = original_path(&request);
    let path = path.as_str();
    let user_ctx = request.extensions().get::<UserContext>().cloned();

    let Some(ctx) = user_ctx else {
        return next.run(request).await;
    };
    if ctx.is_admin || ctx.user_id.as_str().is_empty() {
        return next.run(request).await;
    }

    if is_non_admin_allowed_path(path) {
        next.run(request).await
    } else {
        axum::response::Redirect::to("/admin/profile").into_response()
    }
}

fn is_non_admin_allowed_path(path: &str) -> bool {
    path.starts_with("/admin/profile")
        || path.starts_with("/admin/settings")
        || path.starts_with("/admin/auth/")
        || path.starts_with("/admin/api/")
        || path == "/admin/logout"
        || path == "/admin/login"
        || path == "/admin/register"
        || path == "/admin/add-passkey"
        || path == "/admin/verify-pending"
        || path == "/admin/setup"
        || path == "/admin/demo-register"
        || path == "/admin/"
        || path == "/admin"
}
