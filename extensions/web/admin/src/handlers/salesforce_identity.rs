//! `POST|DELETE /admin/users/{user_id}/salesforce-identity` — set or clear the
//! Salesforce Username an account acts as in one org.
//!
//! The JWT-bearer grant matches its `sub` claim against the Salesforce
//! Username, which is not the login email. Salesforce SSO used to capture it
//! from the `preferred_username` claim; ADFS replaced that login, so the
//! mapping is now an explicit admin act — and this is its audit trail.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::authz;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::salesforce_identity;
use crate::services::connector_oauth::Provider;
use crate::types::UserContext;

fn default_provider() -> String {
    "salesforce".to_owned()
}

#[derive(Debug, Deserialize)]
pub(crate) struct LinkSalesforceIdentityRequest {
    pub sf_username: String,
    #[serde(default = "default_provider")]
    pub provider: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UnlinkSalesforceIdentityQuery {
    #[serde(default = "default_provider")]
    pub provider: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LinkSalesforceIdentityResponse {
    pub user_id: UserId,
    pub provider: String,
    pub sf_username: String,
    pub linked: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UnlinkSalesforceIdentityResponse {
    pub user_id: UserId,
    pub provider: String,
    pub unlinked: bool,
}

// Why: a username for an org this instance does not declare would sign
// assertions nobody can redeem; refuse it at the door.
fn validated_provider(raw: String) -> AdminResult<Provider> {
    let provider = Provider::try_from(raw).map_err(AdminError::BadRequest)?;
    if !provider.is_salesforce() {
        return Err(AdminError::BadRequest(
            "provider must be a Salesforce server id".to_owned(),
        ));
    }
    provider.salesforce_org()?;
    Ok(provider)
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
    let provider = validated_provider(body.provider)?;

    salesforce_identity::upsert_identity(&pool, &user_id, provider.slug(), sf_username).await?;
    authz::salesforce::invalidate(&user_id).await;
    tracing::info!(
        actor = %user_ctx.user_id, user_id = %user_id, provider = provider.slug(),
        "Salesforce identity linked"
    );

    Ok(Json(LinkSalesforceIdentityResponse {
        user_id,
        provider: provider.slug().to_owned(),
        sf_username: sf_username.to_owned(),
        linked: true,
    })
    .into_response())
}

pub(crate) async fn unlink_salesforce_identity_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<UserId>,
    Query(query): Query<UnlinkSalesforceIdentityQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let provider = Provider::try_from(query.provider).map_err(AdminError::BadRequest)?;
    if !provider.is_salesforce() {
        return Err(AdminError::BadRequest(
            "provider must be a Salesforce server id".to_owned(),
        ));
    }

    salesforce_identity::delete_identity(&pool, &user_id, provider.slug()).await?;
    authz::salesforce::invalidate(&user_id).await;
    tracing::info!(
        actor = %user_ctx.user_id, user_id = %user_id, provider = provider.slug(),
        "Salesforce identity unlinked"
    );

    Ok(Json(UnlinkSalesforceIdentityResponse {
        user_id,
        provider: provider.slug().to_owned(),
        unlinked: true,
    })
    .into_response())
}
