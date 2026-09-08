//! `GET /admin/auth/dev/login?code=…` — redeem a one-shot developer login
//! code for an `access_token` session cookie.
//!
//! The route exists only on a development, non-cloud profile: it is not
//! refused elsewhere, it is never mounted, so production has no path to hit.
//! Every failure is the same redirect to `/admin/login?dev=invalid` — a
//! missing, unknown, spent or expired code, or a mint error — so no outcome
//! says whether an account exists. "Spent" allows a short grace after the
//! first redeem, because an address-bar prefetch reaches the link before the
//! person does. The code itself is issued by the
//! `dev-login` CLI extension, which applies the same gate before it writes.

use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::config::ProfileBootstrap;
use systemprompt::models::profile::{Environment, ProfileType};

use super::dev_login_session::{mint_session, session_cookie, session_service};
use crate::repositories::dev_login::consume_dev_login_code;

pub const DEV_LOGIN_PATH: &str = "/admin/auth/dev/login";
const INVALID_REDIRECT: &str = "/admin/login?dev=invalid";

// Why: the one predicate behind the route, the login-page hint and the CLI.
// Development alone is not enough — a cloud profile can carry any
// environment label, and a link that signs someone in without a directory
// must never exist off a developer's own machine.
pub const fn dev_login_allowed(environment: Environment, target: ProfileType) -> bool {
    environment.is_development() && !target.is_cloud()
}

// Why: fails closed. A process that cannot read its profile does not get a
// password-free door on the strength of a default.
pub fn dev_login_enabled() -> bool {
    ProfileBootstrap::get()
        .is_ok_and(|profile| dev_login_allowed(profile.runtime.environment, profile.target))
}

pub fn dev_login_url(api_external_url: &str, code: &str) -> String {
    format!(
        "{}{DEV_LOGIN_PATH}?code={code}",
        api_external_url.trim_end_matches('/')
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct DevLoginParams {
    code: Option<String>,
}

// Why: every outcome is a redirect, success and failure alike; the login page
// is the only place the browser can usefully land.
pub(crate) async fn dev_login_redeem(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    params: Result<Query<DevLoginParams>, QueryRejection>,
) -> Response {
    // Why: lint-ok: http-error — a redirect, not an error envelope.
    let Ok(Query(DevLoginParams { code: Some(code) })) = params else {
        return invalid();
    };

    let user = match consume_dev_login_code(&pool, &code).await {
        Ok(Some(user)) => user,
        Ok(None) => return invalid(),
        Err(e) => {
            tracing::error!(error = %e, "dev login code lookup failed");
            return invalid();
        },
    };

    let service = match session_service(&pool) {
        Ok(service) => service,
        Err(error) => {
            tracing::error!(%error, "dev login service initialization failed");
            return invalid();
        },
    };
    let user_id = user.user_id.clone();
    let email = user.email.clone();
    let (jwt, max_age) = match mint_session(&service, &user, &headers).await {
        Ok(minted) => minted,
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "dev login session mint failed");
            return invalid();
        },
    };

    tracing::info!(user_id = %user_id, email = %email, "dev login code redeemed");

    let mut out = HeaderMap::new();
    if let Ok(val) = session_cookie(&jwt, max_age).parse() {
        out.append(SET_COOKIE, val);
    }
    (out, Redirect::to("/admin")).into_response()
}

// Why: lint-ok: http-error — the failure channel is the login page itself.
fn invalid() -> Response {
    Redirect::to(INVALID_REDIRECT).into_response()
}
