//! Link-based user invites: mint, list, revoke (admin API) and the public
//! accept endpoint that provisions the invitee for passkey enrolment.
//!
//! Scoping: a platform admin may invite into any organization; any other
//! admin only into their own. The accept endpoint is public — the token is
//! the authorization — and returns a `webauthn_setup_tokens` token exactly
//! like passkey self-registration, so the client-side enrolment flow is the
//! same code path.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::oauth::services::webauthn::{generate_setup_token, hash_token};

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::handlers::salesforce_auth::SalesforceDeps;
use crate::repositories::organizations::crud;
use crate::repositories::users::{invites, passkey};
use crate::types::UserContext;

const INVITE_TTL_DAYS: i64 = 7;
const SETUP_TOKEN_TTL_MINUTES: i64 = 10;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateInviteRequest {
    pub email: String,
    /// Organization slug. Optional for org admins (forced to their own);
    /// required for platform admins.
    pub org: Option<String>,
    pub department: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateInviteResponse {
    pub id: String,
    /// Path-only URL — the client prefixes its own origin when copying.
    pub invite_path: String,
    pub email: String,
    pub expires_at: String,
}

pub(crate) async fn create_invite_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateInviteRequest>,
) -> AdminResult<Response> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(AdminError::BadRequest(
            "A valid email address is required".to_owned(),
        ));
    }

    let org = resolve_target_org(&pool, &user_ctx, body.org.as_deref()).await?;
    let department = body
        .department
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .unwrap_or("Default")
        .to_owned();
    let roles = body
        .roles
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| vec!["user".to_owned()]);
    // Why: an invite is not a role-escalation channel — only a platform admin
    // may hand out anything beyond the plain user role.
    if !user_ctx.is_platform_admin && roles.iter().any(|r| r != "user") {
        return Err(AdminError::Forbidden(
            "Only platform administrators may invite with elevated roles".to_owned(),
        ));
    }

    let (raw_token, token_hash) = generate_setup_token();
    let expires_at = Utc::now() + Duration::days(INVITE_TTL_DAYS);
    let id = invites::insert_invite(
        &pool,
        &invites::NewInvite {
            email: &email,
            token_hash: &token_hash,
            org_id: &org.id,
            department: &department,
            roles: &roles,
            invited_by: &user_ctx.user_id,
            expires_at,
        },
    )
    .await?;

    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let entity_id = id.clone();
    let email_for_activity = email.clone();
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_created(
                &uid,
                ActivityEntity::User,
                &entity_id,
                &email_for_activity,
            ),
        )
        .await;
    });

    Ok((
        StatusCode::CREATED,
        Json(CreateInviteResponse {
            id,
            invite_path: format!("/admin/invite/{raw_token}"),
            email,
            expires_at: expires_at.to_rfc3339(),
        }),
    )
        .into_response())
}

#[derive(Debug, Serialize)]
pub(crate) struct PendingInviteView {
    pub id: String,
    pub email: String,
    pub org_name: String,
    pub department: String,
    pub roles: Vec<String>,
    pub expires_at: String,
    pub created_at: String,
}

pub(crate) async fn list_invites_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let org_filter = org_filter_for(&pool, &user_ctx).await?;
    let rows = invites::list_pending_invites(&pool, org_filter.as_deref()).await?;
    let body: Vec<PendingInviteView> = rows
        .into_iter()
        .map(|r| PendingInviteView {
            id: r.id,
            email: r.email,
            org_name: r.org_name,
            department: r.department,
            roles: r.roles,
            expires_at: r.expires_at.to_rfc3339(),
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();
    Ok(Json(body).into_response())
}

pub(crate) async fn revoke_invite_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(invite_id): Path<String>,
) -> AdminResult<Response> {
    let org_filter = org_filter_for(&pool, &user_ctx).await?;
    if !invites::revoke_invite(&pool, &invite_id, org_filter.as_deref()).await? {
        return Err(AdminError::NotFound(
            "No pending invite with that id".to_owned(),
        ));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

// Why: platform admins see and revoke every org's invites (`None` filter);
// other admins are confined to their own organization's.
async fn org_filter_for(
    pool: &PgPool,
    user_ctx: &UserContext,
) -> Result<Option<String>, AdminError> {
    if user_ctx.is_platform_admin {
        return Ok(None);
    }
    let slug = crud::find_organization_for_user(pool, &user_ctx.user_id)
        .await?
        .ok_or_else(|| {
            AdminError::Forbidden(
                "You are not a member of an organization, so there are no invites to manage."
                    .to_owned(),
            )
        })?;
    let org = crud::find_organization_by_slug(pool, &slug)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("No organization with slug '{slug}'.")))?;
    Ok(Some(org.id))
}

async fn resolve_target_org(
    pool: &PgPool,
    user_ctx: &UserContext,
    requested: Option<&str>,
) -> Result<crud::OrganizationSummary, AdminError> {
    let slug = if user_ctx.is_platform_admin {
        requested
            .map(str::to_owned)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AdminError::BadRequest("An organization slug is required".to_owned()))?
    } else {
        // Why: `?org` from a non-platform admin is ignored, not rejected — the
        // UI never sends it for them, and honouring it would cross tenants.
        crud::find_organization_for_user(pool, &user_ctx.user_id)
            .await?
            .ok_or_else(|| {
                AdminError::Forbidden(
                    "You are not a member of an organization, so you cannot invite users."
                        .to_owned(),
                )
            })?
    };
    crud::find_organization_by_slug(pool, &slug)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("No organization with slug '{slug}'.")))
}

#[derive(Debug, Deserialize)]
pub(crate) struct AcceptInviteRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AcceptInviteResponse {
    pub email: String,
    pub setup_token: String,
    pub expires_in_seconds: i64,
}

// Why: public — the invite token is the authorization. It exchanges a valid
// token for a passkey setup token, provisioning the account as it goes; once
// the invite is marked accepted the invitee signs in with their passkey.
pub(crate) async fn accept_invite_handler(
    Extension(deps): Extension<SalesforceDeps>,
    Json(req): Json<AcceptInviteRequest>,
) -> AdminResult<Json<AcceptInviteResponse>> {
    let token = req.token.trim();
    if token.is_empty() {
        return Err(AdminError::BadRequest(
            "An invite token is required".to_owned(),
        ));
    }
    let token_hash = hash_token(token);
    let invite = invites::find_valid_invite_by_hash(&deps.write_pool, &token_hash)
        .await?
        .ok_or_else(|| {
            AdminError::NotFound(
                "This invite link is invalid, expired, or already used.".to_owned(),
            )
        })?;

    let user_id = invites::accept_invite_and_provision(&deps.write_pool, &invite).await?;
    crate::authz::organization::invalidate(&user_id).await;

    let (raw_token, setup_hash) = generate_setup_token();
    let expires_at = Utc::now() + Duration::minutes(SETUP_TOKEN_TTL_MINUTES);
    passkey::insert_setup_token(&deps.write_pool, &user_id, &setup_hash, expires_at).await?;

    tracing::info!(user_id = %user_id, email = %invite.email, "Invite accepted; setup token issued");

    Ok(Json(AcceptInviteResponse {
        email: invite.email,
        setup_token: raw_token,
        expires_in_seconds: SETUP_TOKEN_TTL_MINUTES * 60,
    }))
}
