//! Keeps console readers from invoking mutations mounted beside SSR pages.

use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::types::UserContext;

pub(super) async fn require_write_access(request: Request, next: Next) -> Response {
    let read = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let admin = request
        .extensions()
        .get::<UserContext>()
        .is_some_and(|user| user.is_admin);
    let path = request.uri().path();
    // Why: these endpoints operate exclusively on the authenticated caller;
    // their handlers enforce ownership and must remain usable by ordinary users.
    let own_account = matches!(
        path,
        "/api/profile/bridge-code" | "/api/profile/salesforce/unlink"
    ) || path == "/devices/pats"
        || path.starts_with("/devices/pats/")
        || path.starts_with("/devices/certs/")
        || path == "/tokens/pats"
        || path.starts_with("/tokens/pats/");
    if read || admin || own_account {
        next.run(request).await
    } else {
        (StatusCode::FORBIDDEN, "Admin access required").into_response()
    }
}
