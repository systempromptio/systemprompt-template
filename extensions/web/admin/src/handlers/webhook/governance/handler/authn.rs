//! Bearer-token authentication for the governance webhook.
//!
//! Validates the inbound JWT against the hook/plugin/api audiences and maps
//! failures to a denied [`Decision`] plus a recorded auth-denial audit row.

use axum::{http::HeaderMap, response::Response};
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::OauthError;
use systemprompt_security::authz::{Decision, DenyReason};

use crate::handlers::webhook::helpers::{extract_bearer_token, get_jwt_issuer};

use super::super::types::AuthDenialParams;
use super::{build_response, spawn_auth_denial};

pub(super) fn deny_for_auth_failure(reason: &str) -> Decision {
    Decision::Deny {
        reason: DenyReason::HookUnavailable {
            policy: format!("auth_failure: {reason}"),
        },
    }
}

pub(super) fn authenticate_request(
    headers: &HeaderMap,
    denial_params: &AuthDenialParams<'_>,
) -> Result<UserId, Box<Response>> {
    let Some(token) = extract_bearer_token(headers) else {
        let reason = "Missing Authorization header — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        return Err(Box::new(build_response(&deny_for_auth_failure(reason))));
    };

    let jwt_issuer = match get_jwt_issuer() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            let reason = "Internal configuration error — tool call blocked";
            spawn_auth_denial(denial_params, reason);
            return Err(Box::new(build_response(&deny_for_auth_failure(reason))));
        }
    };

    let expected_aud = "hook|plugin|api";
    let claims = systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
            JwtAudience::Api,
        ],
    )
    .map_err(|e| {
        log_jwt_failure(&e, expected_aud, &jwt_issuer);
        let reason = "Invalid or expired token — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        Box::new(build_response(&deny_for_auth_failure(reason)))
    })?;

    Ok(UserId::new(&claims.sub))
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
            "missing kid header".to_string(),
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
