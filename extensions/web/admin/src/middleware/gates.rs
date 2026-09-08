//! Authentication and authorisation layers for the admin plane.
//!
//! These sit above `user_context_middleware`, which is what populates the
//! [`UserContext`] they read. They are separated from page context because
//! they answer a different question: context decides what a page renders,
//! these decide whether the request is allowed to reach one at all.

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::handlers::shared::ErrorBody;
use crate::types::{Role, UserContext, has_any};

// Why: `nest_service` strips its prefix from `request.uri()`, so a layer
// inside the admin SSR router sees `/profile` for a request to
// `/admin/profile`; matching user-facing paths requires `OriginalUri`.
fn original_path(request: &Request) -> String {
    request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map_or_else(
            || request.uri().path().to_owned(),
            |o| o.0.path().to_owned(),
        )
}

fn original_target(request: &Request) -> String {
    fn render(uri: &axum::http::Uri) -> String {
        uri.path_and_query()
            .map_or_else(|| uri.path().to_owned(), ToString::to_string)
    }
    request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map_or_else(|| render(request.uri()), |o| render(&o.0))
}

pub(crate) async fn require_user_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => {
            let target = urlencoding::encode(&original_target(&request)).into_owned();
            let redirect_url = format!("/admin/login?redirect={target}");
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


// Why: anonymous users are deliberately not handled here —
// `require_user_middleware` runs after this layer and owns that case. The path
// must come from `OriginalUri`: this layer sits inside a
// `nest_service("/admin", …)`, which strips the prefix, and every arm of
// `is_non_admin_allowed_path` matches on the full path.
pub(crate) async fn non_admin_gate_middleware(request: Request, next: Next) -> Response {
    let path = original_path(&request);
    let path = path.as_str();
    let user_ctx = request.extensions().get::<UserContext>().cloned();

    let Some(ctx) = user_ctx else {
        return next.run(request).await;
    };
    if ctx.is_console || ctx.user_id.as_str().is_empty() {
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
        || path.starts_with("/admin/history")
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

pub(crate) async fn require_roles_middleware(
    State(accepted): State<&'static [Role]>,
    request: Request,
    next: Next,
) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    let allowed = user_ctx.is_some_and(|ctx| has_any(&ctx.roles, accepted));
    if allowed {
        return next.run(request).await;
    }
    let names = accepted
        .iter()
        .map(|r| r.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    (
        StatusCode::FORBIDDEN,
        axum::Json(ErrorBody {
            error: format!("Role required: {names}"),
        }),
    )
        .into_response()
}
