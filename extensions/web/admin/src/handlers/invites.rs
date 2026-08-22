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
use systemprompt::oauth::services::webauthn::generate_setup_token;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::repositories::organizations::crud;
use crate::repositories::users::invites;
use crate::types::UserContext;

const INVITE_TTL_DAYS: i64 = 7;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateInviteRequest {
    pub email: String,
    pub org: Option<String>,
    pub department: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateInviteResponse {
    pub id: String,
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

// Why: the raw token is shown exactly once at mint time, so a lost link is
// recovered by revoking and reminting in one transaction, never by re-reading.
// Same response shape as create, so the client's copy affordance is shared.
pub(crate) async fn regenerate_invite_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(invite_id): Path<String>,
) -> AdminResult<Response> {
    let org_filter = org_filter_for(&pool, &user_ctx).await?;
    let (raw_token, token_hash) = generate_setup_token();
    let expires_at = Utc::now() + Duration::days(INVITE_TTL_DAYS);
    let Some(new_id) = invites::insert_regenerated_invite(
        &pool,
        &invite_id,
        org_filter.as_deref(),
        &token_hash,
        expires_at,
    )
    .await?
    else {
        return Err(AdminError::NotFound(
            "No pending invite with that id".to_owned(),
        ));
    };

    let row = invites::list_pending_invites(&pool, org_filter.as_deref())
        .await?
        .into_iter()
        .find(|r| r.id == new_id);
    let email = row.map_or_else(String::new, |r| r.email);

    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let entity_id = new_id.clone();
    let email_for_activity = email.clone();
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_updated(
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
            id: new_id,
            invite_path: format!("/admin/invite/{raw_token}"),
            email,
            expires_at: expires_at.to_rfc3339(),
        }),
    )
        .into_response())
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
