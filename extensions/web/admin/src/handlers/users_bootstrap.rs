//! `POST /users` — operator-created accounts and their credential bootstrap.
//!
//! Split from `users.rs` at the 300-line ceiling. A user created here gets no
//! credential of its own: Systemprompt SSO is the only door, so the row waits to be
//! adopted by the first assertion carrying the matching email and a mapped
//! Active Directory group.

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

// Why: there is no link to hand back — the note tells the operator what to say
// instead.
#[derive(Debug, serde::Serialize)]
pub(crate) struct CreatedUserResponse {
    pub user: crate::types::UserSummary,
    pub invite_note: String,
}

pub(crate) async fn create_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let user = repositories::users::mutations::create_user(&pool, &body).await?;

    let invite_note = sign_in_note();

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
        Json(CreatedUserResponse { user, invite_note }),
    )
        .into_response())
}

// Why: an admin-created row needs no sign-in link. Systemprompt SSO is the only
// door, and with `auto_provision` on the AD group is the invitation — the
// account created here is adopted by the first assertion carrying a mapped
// group and the matching email, which is also what places them in their
// groups and projects.
fn sign_in_note() -> String {
    "User created. They sign in with Systemprompt SSO — there is no link to send. Their groups, \
     projects and roles follow their Active Directory group membership."
        .to_owned()
}
