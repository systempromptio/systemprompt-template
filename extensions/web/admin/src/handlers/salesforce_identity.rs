//! `POST|DELETE /admin/users/{user_id}/salesforce-identity` — set or clear the
//! Salesforce Username an account acts as.
//!
//! The JWT-bearer grant matches its `sub` claim against the Salesforce
//! Username, which is not the login email. Salesforce SSO used to capture it
//! from the `preferred_username` claim; ADFS replaced that login, so the
//! mapping is now an explicit admin act — and this is its audit trail.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::authz;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::salesforce_identity;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct LinkSalesforceIdentityRequest {
    pub sf_username: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LinkSalesforceIdentityResponse {
    pub user_id: UserId,
    pub sf_username: String,
    pub linked: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UnlinkSalesforceIdentityResponse {
    pub user_id: UserId,
    pub unlinked: bool,
}

// Why: Salesforce Usernames are formatted as email addresses but are not
// deliverable addresses, so the only checks worth making are that one was sent
// and that it looks like the Username form. An empty value would sign
// assertions with an empty `sub` and fail opaquely at Salesforce.
fn validated_sf_username(raw: &str) -> AdminResult<&str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AdminError::BadRequest(
            "sf_username must not be empty".to_owned(),
        ));
    }
    if !trimmed.contains('@') {
        return Err(AdminError::BadRequest(
            "sf_username must be a Salesforce Username, which is email-shaped".to_owned(),
        ));
    }
    Ok(trimmed)
}

pub(crate) async fn link_salesforce_identity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<UserId>,
    Json(body): Json<LinkSalesforceIdentityRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let sf_username = validated_sf_username(&body.sf_username)?;

    salesforce_identity::upsert_identity(&pool, &user_id, sf_username).await?;
    authz::salesforce::invalidate(&user_id).await;
    tracing::info!(
        actor = %user_ctx.user_id, user_id = %user_id,
        "Salesforce identity linked"
    );

    Ok(Json(LinkSalesforceIdentityResponse {
        user_id,
        sf_username: sf_username.to_owned(),
        linked: true,
    })
    .into_response())
}

pub(crate) async fn unlink_salesforce_identity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<UserId>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    salesforce_identity::delete_identity(&pool, &user_id).await?;
    authz::salesforce::invalidate(&user_id).await;
    tracing::info!(
        actor = %user_ctx.user_id, user_id = %user_id,
        "Salesforce identity unlinked"
    );

    Ok(Json(UnlinkSalesforceIdentityResponse {
        user_id,
        unlinked: true,
    })
    .into_response())
}
