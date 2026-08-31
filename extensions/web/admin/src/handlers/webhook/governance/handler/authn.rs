//! Bearer-token authentication for the governance webhook.
//!
//! Validates the inbound JWT against the hook/plugin/api audiences and maps
//! failures to a denied [`Decision`] plus a recorded auth-denial audit row.

use axum::http::HeaderMap;
use axum::response::Response;
use systemprompt::identifiers::{ClientId, UserId};
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::OauthError;
use systemprompt_security::authz::{Decision, DenyReason};
use systemprompt_security::policy::types::AccessScope;

use crate::handlers::webhook::helpers::{extract_bearer_token, get_jwt_issuer};

use super::super::scope::scope_from_permissions;
use super::super::types::AuthDenialParams;
use super::{build_response, spawn_auth_denial};

pub(super) struct Principal {
    pub user_id: UserId,
    pub token_scope: AccessScope,
    pub client_id: Option<ClientId>,
}

pub(super) fn deny_for_auth_failure(reason: &str) -> Decision {
    // Why: core 0.42.0 split the cause out of `policy` into `detail`, so an
    // audit row can tell a transient fault from a rejected token. The reason
    // was being smuggled into `policy` as "auth_failure: <reason>", which made
    // every distinct failure a distinct policy name and nothing groupable.
    Decision::Deny {
        reason: DenyReason::HookUnavailable {
            policy: "auth_failure".to_owned(),
            detail: reason.to_owned(),
        },
    }
}

// Why: the `Err` arm is deliberately **not** an
// [`crate::error::AdminError`]. A `PreToolUse` hook blocks a tool call by
// answering `200 OK` with a deny decision; a `401` is a transport failure,
// which the client is free to treat as the hook being unavailable and carry on.
// Converting this channel to the admin error type would silently turn every
// rejected token into an allowed tool call, so it stays a pre-built
// [`Response`].
pub(super) fn authenticate_request(
    headers: &HeaderMap,
    denial_params: &AuthDenialParams<'_>,
) -> Result<Principal, Box<Response>> {
    let Some(token) = extract_bearer_token(headers) else {
        let reason = "Missing Authorization header — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        return Err(Box::new(build_response(
            &deny_for_auth_failure(reason),
            denial_params.hook_event_name,
        )));
    };

    let jwt_issuer = match get_jwt_issuer() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            let reason = "Internal configuration error — tool call blocked";
            spawn_auth_denial(denial_params, reason);
            return Err(Box::new(build_response(
                &deny_for_auth_failure(reason),
                denial_params.hook_event_name,
            )));
        },
    };

    let expected_aud = "hook|plugin|api";
    let claims = systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_owned()),
            JwtAudience::Resource("plugin".to_owned()),
            JwtAudience::Api,
        ],
    )
    .map_err(|e| {
        log_jwt_failure(&e, expected_aud, &jwt_issuer);
        let reason = "Invalid or expired token — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        Box::new(build_response(
            &deny_for_auth_failure(reason),
            denial_params.hook_event_name,
        ))
    })?;

    Ok(Principal {
        user_id: UserId::new(&claims.sub),
        token_scope: scope_from_permissions(claims.permissions()),
        client_id: claims.client_id.clone(),
    })
}

fn log_jwt_failure(err: &OauthError, expected_aud: &str, issuer: &str) {
    let (detail, message) = jwt_failure_detail(err);
    tracing::warn!(detail = %detail, expected_aud, issuer, "{}", message);
}

fn jwt_failure_detail(err: &OauthError) -> (String, &'static str) {
    match err {
        OauthError::TokenAlgMismatch { got, expected } => (
            format!("alg got={got} expected={expected}"),
            "Governance webhook JWT rejected: signing algorithm mismatch",
        ),
        OauthError::TokenMissingKid => (
            "missing kid header".to_owned(),
            "Governance webhook JWT rejected: missing `kid` header",
        ),
        OauthError::TokenUnknownKid { kid } => (
            format!("unknown kid={kid}"),
            "Governance webhook JWT rejected: unknown signing key — token was minted under a \
             different RSA authority",
        ),
        OauthError::Expired(reason) => (
            format!("expired: {reason}"),
            "Governance webhook JWT rejected: token expired",
        ),
        other => (
            format!("{other}"),
            "Governance webhook JWT validation failed",
        ),
    }
}
