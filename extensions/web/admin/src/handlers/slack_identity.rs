//! `POST|DELETE /admin/users/{user_id}/slack-identity` — attach or detach the
//! Slack account an admin drives the platform from.
//!
//! Inbound Slack normally links a sender to their systemprompt account by
//! workspace email (`authz.link_by_workspace_email`). This is the escape hatch
//! for the accounts that cannot take that path — a Slack profile carrying an
//! unconfirmed or different address — and the audit trail for who was granted
//! chat access by hand.
//!
//! The mapping is the same `federated_identities` row the Salesforce Connect
//! flow writes, under Slack's issuer, so a linked sender resolves straight to
//! this user and inherits their roles.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories::users::federated;
use crate::types::UserContext;

// Why: The issuer core namespaces Slack senders under — see the messaging
// pipeline's `ISSUER` constant. A mismatch here would write a row nothing
// ever reads.
const SLACK_ISSUER: &str = "https://slack.com";

#[derive(Debug, Deserialize)]
pub(crate) struct LinkSlackIdentityRequest {
    pub slack_user_id: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LinkSlackIdentityResponse {
    pub user_id: UserId,
    pub slack_user_id: String,
    pub linked: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UnlinkSlackIdentityResponse {
    pub user_id: UserId,
    pub removed: u64,
}

// Why: Slack ids are opaque workspace-side strings, so the only check worth
// making is that one was actually sent — an empty external_sub would map every
// unlinked sender onto this account.
fn validated_slack_user_id(raw: &str) -> AdminResult<&str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AdminError::BadRequest(
            "slack_user_id must not be empty".to_owned(),
        ));
    }
    Ok(trimmed)
}

pub(crate) async fn link_slack_identity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<UserId>,
    Json(body): Json<LinkSlackIdentityRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let slack_user_id = validated_slack_user_id(&body.slack_user_id)?;

    let outcome =
        federated::link_identity_to_user(&pool, SLACK_ISSUER, slack_user_id, &user_id).await?;
    if outcome == federated::LinkOutcome::AlreadyLinkedElsewhere {
        return Err(AdminError::Conflict(
            "That Slack account is already linked to another user".to_owned(),
        ));
    }

    tracing::info!(
        actor = %user_ctx.user_id,
        user_id = %user_id,
        slack_user_id,
        "Slack identity linked"
    );
    Ok(Json(LinkSlackIdentityResponse {
        user_id,
        slack_user_id: slack_user_id.to_owned(),
        linked: true,
    })
    .into_response())
}

pub(crate) async fn unlink_slack_identity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<UserId>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    let removed =
        federated::delete_federated_identities_for_issuer(&pool, &user_id, SLACK_ISSUER).await?;

    tracing::info!(
        actor = %user_ctx.user_id,
        user_id = %user_id,
        removed,
        "Slack identity unlinked"
    );
    Ok(Json(UnlinkSlackIdentityResponse { user_id, removed }).into_response())
}
