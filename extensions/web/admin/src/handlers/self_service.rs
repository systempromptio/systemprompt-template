//! The two writes a person makes to their own account.
//!
//! Everything else under `/api/public/admin` is role-gated, because it mutates
//! somebody else's identity or the instance's configuration. These two are not:
//! they are scoped to the caller, and the target is read from the validated
//! session rather than from the request, so there is no parameter to point at
//! another account. That is why they sit in their own module and their own
//! router tier instead of being widened into the admin one.

use std::sync::Arc;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::repositories::users::user_settings::UserSettingsInput;
use crate::types::UserContext;

// Why: shape, not membership. The value is stored and then used to render every
// timestamp the console shows this person, so an unbounded string would be
// written once and misread forever. Checking it against the IANA registry would
// mean a new dependency to hold a list the browser already has — the picker is
// built from `Intl.supportedValuesOf('timeZone')` — so this rejects what an
// IANA name can never look like and leaves the enumeration to the client.
const TIMEZONE_MAX: usize = 64;

fn validated(input: &UserSettingsInput) -> Result<(), AdminError> {
    let tz = input.timezone.trim();
    let shaped = !tz.is_empty()
        && tz.len() <= TIMEZONE_MAX
        && tz
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '+'));
    if !shaped {
        return Err(AdminError::BadRequest(format!(
            "{tz:?} is not shaped like an IANA timezone name."
        )));
    }
    for (field, value) in [
        ("display_name", input.display_name.as_deref()),
        ("avatar_url", input.avatar_url.as_deref()),
    ] {
        if value.is_some_and(|v| v.chars().count() > 512) {
            return Err(AdminError::BadRequest(format!("{field} is too long.")));
        }
    }
    Ok(())
}

pub(crate) async fn update_own_settings_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    body: Result<Json<UserSettingsInput>, JsonRejection>,
) -> AdminResult<Response> {
    let Json(input) = body.map_err(|e| AdminError::BadRequest(e.body_text()))?;
    validated(&input)?;
    let saved =
        repositories::users::user_settings::update_user_settings(&pool, &user_ctx.user_id, &input)
            .await?;
    Ok(Json(saved).into_response())
}

// Why: the caller has to name the account they are closing. A bodyless DELETE
// on this path would otherwise be a working delete, and there is no undo — any
// crawler, any route prober, any mistyped curl closes an account. Echoing the
// session's own email costs a person one paste behind a dialog they already
// answered, and it is the difference between a destructive route that can be
// hit by accident and one that cannot.
#[derive(Debug, Deserialize)]
pub(crate) struct DeleteAccountRequest {
    confirm_email: String,
}

// Why: irreversible, and scoped to the caller — `user_ctx.user_id` comes from
// the session, so this can only ever delete the account that asked. The
// settings row goes first: it carries no foreign key to `users`, so deleting
// the account alone would orphan it. The audit spine does not go: `ai_requests`
// holds no foreign key either, so closing an account does not erase what the
// gateway recorded about it.
pub(crate) async fn delete_own_account_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    body: Result<Json<DeleteAccountRequest>, JsonRejection>,
) -> AdminResult<Response> {
    let Json(request) = body.map_err(|e| {
        AdminError::BadRequest(format!(
            "Deleting an account needs {{\"confirm_email\": \"…\"}} naming it: {}",
            e.body_text()
        ))
    })?;
    if !request
        .confirm_email
        .eq_ignore_ascii_case(user_ctx.email.as_str())
    {
        return Err(AdminError::BadRequest(
            "confirm_email does not match the signed-in account.".to_owned(),
        ));
    }
    repositories::users::user_settings::delete_user_settings(&pool, &user_ctx.user_id).await?;
    if !repositories::users::mutations::delete_user(&pool, &user_ctx.user_id).await? {
        return Err(AdminError::NotFound("No such user.".to_owned()));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}
