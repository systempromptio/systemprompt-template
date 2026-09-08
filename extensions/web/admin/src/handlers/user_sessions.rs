//! HTTP handlers for listing and revoking a user's sign-in sessions.
//!
//! Splitting this from [`crate::handlers::users`] keeps session lifecycle —
//! the only place this surface writes `user_sessions` — in one file rather
//! than buried among identity CRUD.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::sessions;
use crate::types::UserContext;

#[derive(Debug, serde::Serialize)]
pub(crate) struct SessionsListResponse {
    pub sessions: Vec<sessions::SigninSessionRow>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RevokedResponse {
    pub revoked: u64,
}

pub(crate) async fn list_user_sessions_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()));
    }
    let sessions = sessions::list_signin_sessions(&pool, &user_id).await?;
    Ok(Json(SessionsListResponse { sessions }).into_response())
}

pub(crate) async fn revoke_user_session_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path((user_id_raw, session_id_raw)): Path<(String, String)>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    let session_id = SessionId::new(session_id_raw);
    guard(&user_ctx)?;

    if !sessions::revoke_signin_session(&pool, &user_id, &session_id).await? {
        return Err(AdminError::NotFound(
            "No live session with that id for this user".to_owned(),
        ));
    }
    record(&pool, &user_ctx, &user_id, 1);
    Ok(Json(RevokedResponse { revoked: 1 }).into_response())
}

pub(crate) async fn revoke_all_user_sessions_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    guard(&user_ctx)?;

    // Why: no 404 on zero. "Sign this person out everywhere" has succeeded
    // when they hold no live session, however many rows that took; only a
    // single named session can be genuinely absent.
    let revoked = sessions::revoke_all_signin_sessions(&pool, &user_id).await?;
    if revoked > 0 {
        record(&pool, &user_ctx, &user_id, revoked);
    }
    Ok(Json(RevokedResponse { revoked }).into_response())
}

// Why: admins administer every account on this single-tenant instance;
// nobody else administers anyone.
fn guard(user_ctx: &UserContext) -> Result<(), AdminError> {
    // Why: `admin`, not `is_console`. Revoking another account's sessions is
    // a credential mutation.
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    Ok(())
}

fn record(pool: &Arc<PgPool>, user_ctx: &UserContext, target: &UserId, count: u64) {
    let pool = Arc::clone(pool);
    let actor = user_ctx.user_id.clone();
    let target = target.clone();
    tokio::spawn(async move {
        activity::record(
            &pool,
            NewActivity::entity_updated(
                &actor,
                ActivityEntity::User,
                target.as_str(),
                &format!("revoked {count} session(s)"),
            ),
        )
        .await;
    });
}
