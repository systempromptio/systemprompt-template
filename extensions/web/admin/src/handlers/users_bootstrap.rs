//! `POST /users` — operator-created accounts and their credential bootstrap.
//!
//! Split from `users.rs` at the 300-line ceiling. A user created here has no
//! passkey and no setup token, so the create path also mints an invite: the
//! accept flow adopts an existing account by email, which makes the invite
//! link a working first sign-in for a row that already exists.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::types::{CreateUserRequest, UserContext};

// Why: a directly created account has no credential at all, so the response
// carries a 7-day invite link the operator hands the person to enrol a
// passkey. Null with a note when the email's domain claims no organization
// (invites require one) or a pending invite already exists.
#[derive(Debug, serde::Serialize)]
pub(crate) struct CreatedUserResponse {
    pub user: crate::types::UserSummary,
    pub invite_path: Option<String>,
    pub invite_note: Option<String>,
    pub expires_at: Option<String>,
}

pub(crate) async fn create_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    // Why: the same escalation rule as invites — creating a colleague with an
    // elevated role is a platform-admin act, not an org-admin one.
    if !user_ctx.is_platform_admin && body.roles.iter().any(|r| r != "user") {
        return Err(AdminError::Forbidden(
            "Only platform administrators may create users with elevated roles".to_owned(),
        ));
    }
    let created = repositories::users::mutations::create_user(&pool, &body).await?;
    let user = created.summary;

    let (invite_path, invite_note, expires_at) =
        mint_bootstrap_invite(&pool, &user_ctx, &body, created.org_id.as_deref()).await;

    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let new_user_id = user.user_id.clone();
    let name = user
        .display_name
        .clone()
        .unwrap_or_else(|| user.user_id.as_str().to_owned());
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_created(&uid, ActivityEntity::User, new_user_id.as_str(), &name),
        )
        .await;
    });
    Ok((
        StatusCode::CREATED,
        Json(CreatedUserResponse {
            user,
            invite_path,
            invite_note,
            expires_at,
        }),
    )
        .into_response())
}

// Why: best-effort by design — the account was already created, so a failed
// invite mint degrades to a note, never a failed request. The accept flow
// adopts an existing account by email, so the link doubles as passkey
// bootstrap for the row created above.
async fn mint_bootstrap_invite(
    pool: &PgPool,
    user_ctx: &UserContext,
    body: &CreateUserRequest,
    org_id: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    use systemprompt::oauth::services::webauthn::generate_setup_token;

    let Some(org_id) = org_id else {
        return (
            None,
            Some(
                "No organization claims this email domain, so no sign-in link was minted —                  the user can sign in via SSO, or mint an invite after claiming the domain."
                    .to_owned(),
            ),
            None,
        );
    };
    let (raw_token, token_hash) = generate_setup_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
    let department = body
        .department
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .unwrap_or("Default");
    let result = repositories::users::invites::insert_invite(
        pool,
        &repositories::users::invites::NewInvite {
            email: body.email.as_str(),
            token_hash: &token_hash,
            org_id,
            department,
            roles: &body.roles,
            invited_by: &user_ctx.user_id,
            expires_at,
        },
    )
    .await;
    match result {
        Ok(_) => (
            Some(format!("/admin/invite/{raw_token}")),
            None,
            Some(expires_at.to_rfc3339()),
        ),
        Err(e) => (
            None,
            Some(format!("User created, but no sign-in link was minted: {e}")),
            None,
        ),
    }
}
