//! `GET /admin/auth/adfs/start` — build an `AuthnRequest` and redirect the
//! browser to the AD FS SSO endpoint (HTTP-Redirect binding), stashing the
//! request id and an anti-CSRF `RelayState` in a cookie for the callback.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::Extension;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use saml::{Binding, Dispatch, StartLogin};
use serde::Deserialize;

use super::provider::{idp_descriptor, service_provider};
use super::{
    AdfsDeps, FlowState, login_error, random_url_safe, sanitize_redirect, state_cookie, use_https,
};

#[derive(Deserialize)]
pub(crate) struct StartParams {
    redirect: Option<String>,
}

pub(crate) async fn adfs_start(
    Extension(deps): Extension<AdfsDeps>,
    Query(params): Query<StartParams>,
) -> Response {
    // Why: lint-ok: http-error — an SSO flow reports failure by redirecting back to
    // the login page with ?sso=<reason>; an error status would strand the
    // browser on a dead end instead of returning the user to a usable page.
    let cfg = &deps.config;
    if !cfg.is_usable() {
        return login_error("unavailable");
    }
    let (idp, sp) = match (idp_descriptor(cfg).await, service_provider(cfg)) {
        (Ok(idp), Ok(sp)) => (idp, sp),
        (Err(e), _) | (_, Err(e)) => {
            tracing::error!(error = %e, "ADFS SSO cannot start");
            return login_error("unavailable");
        },
    };

    let state_token = random_url_safe();
    let started = match sp.start_login(
        &idp,
        StartLogin {
            relay_state: Some(&state_token),
            binding: Binding::HttpRedirect,
            force_authn: false,
            is_passive: false,
            requested_name_id_format: None,
            requested_authn_context: None,
            acs_index: None,
            acs_url: None,
            response_binding: None,
        },
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "ADFS AuthnRequest could not be built");
            return login_error("unavailable");
        },
    };
    let Dispatch::Redirect(target) = started.dispatch else {
        tracing::error!("ADFS start produced a POST dispatch for a redirect binding");
        return login_error("unavailable");
    };

    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let redirect_to = sanitize_redirect(params.redirect);

    let flow = FlowState {
        state: state_token,
        request_id: started.tracker.request_id,
        issued_at_unix: issued_at,
        redirect_to,
    };

    let mut headers = HeaderMap::new();
    if let Ok(val) = state_cookie(&flow, use_https()).parse() {
        headers.insert(SET_COOKIE, val);
    }
    (headers, Redirect::to(target.as_str())).into_response()
}
