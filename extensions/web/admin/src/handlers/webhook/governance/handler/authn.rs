//! Bearer-token authentication for the governance webhook.
//!
//! Validates the inbound JWT against the hook/plugin/api audiences and maps
//! failures to a denied [`Decision`] plus a recorded auth-denial audit row.

use crate::env::AdminEnv;

use axum::http::HeaderMap;
use axum::response::Response;
use systemprompt::governance::authz::{Decision, DenyReason};
use systemprompt::identifiers::{ClientId, JwtToken, UserId};
use systemprompt::models::auth::JwtAudience;

use crate::handlers::webhook::helpers::{WEBHOOK_AUDIENCES, extract_bearer_token};

use super::super::types::AuthDenialParams;
use super::{build_response, spawn_auth_denial};

pub(super) struct Principal {
    pub user_id: UserId,
    pub client_id: Option<ClientId>,
}

// Why: `policy` names the plane and `detail` carries the cause — core added
// `detail` precisely so a transient fault and a malformed token stop writing
// byte-identical audit rows. Folding the cause into `policy`, as this did
// before the field existed, would keep every failure looking like a distinct
// policy and leave the audit column unqueryable.
pub(super) fn deny_for_auth_failure(reason: &str) -> Decision {
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
pub(super) async fn authenticate_request(
    env: &AdminEnv,
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

    let mut audiences = WEBHOOK_AUDIENCES.to_vec();
    audiences.push(JwtAudience::Resource("plugin".to_owned()));
    let (context, _) = env
        .authenticator()
        .decode_for_audiences(&JwtToken::new(token), &audiences)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, expected_aud = "hook|plugin|api", "Governance webhook JWT rejected");
            let reason = "Invalid or expired token — tool call blocked";
            spawn_auth_denial(denial_params, reason);
            Box::new(build_response(
                &deny_for_auth_failure(reason),
                denial_params.hook_event_name,
            ))
        })?;

    Ok(Principal {
        user_id: context.user_id,
        client_id: context.client_id,
    })
}
