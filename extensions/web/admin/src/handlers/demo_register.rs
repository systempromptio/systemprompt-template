//! Self-service demo account registration.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::repositories::users::magic_links;
use crate::types::{CreateUserRequest, UserContext};


#[derive(Deserialize, Debug)]
pub(crate) struct DemoRegisterRequest {
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DemoRegisterResponse {
    pub ok: bool,
    pub registration_url: String,
    pub user_id: UserId,
    pub display_name: String,
}

/// Derive a stable, URL-safe `UserId` from an email's local part, falling back
/// to `"user"` when sanitisation leaves nothing usable.
fn derive_user_id(email_str: &str) -> UserId {
    let local_part = email_str.split('@').next().unwrap_or("user");
    let sanitized = local_part
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .trim_matches('-')
        .to_owned();
    UserId::new(if sanitized.is_empty() {
        "user".to_owned()
    } else {
        sanitized
    })
}

pub(crate) async fn create_demo_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<DemoRegisterRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    let email_str = body.email.trim().to_lowercase();
    let name = body.name.trim().to_owned();

    if email_str.is_empty() || !email_str.contains('@') {
        return Err(AdminError::BadRequest("Invalid email address".to_owned()));
    }
    if name.is_empty() {
        return Err(AdminError::BadRequest("Name is required".to_owned()));
    }

    let email = Email::try_new(email_str.clone())
        .map_err(|e| AdminError::BadRequest(format!("Invalid email address: {e}")))?;

    let user_id = derive_user_id(&email_str);

    let roles = match body.role.as_str() {
        "admin" => vec!["admin".to_owned()],
        _ => vec![],
    };

    let create_req = CreateUserRequest {
        user_id: user_id.clone(),
        display_name: name.clone(),
        email,
        roles,
        status: Some("active".to_owned()),
    };

    repositories::users::mutations::create_user(&pool, &create_req).await?;

    let raw_token = magic_links::create_magic_link_token(&pool, &email_str, None)
        .await
        .map_err(AdminError::internal)?;

    let registration_url = format!("/admin/add-passkey?token={raw_token}");

    Ok((
        StatusCode::OK,
        Json(DemoRegisterResponse {
            ok: true,
            registration_url,
            user_id: user_id.clone(),
            display_name: name,
        }),
    )
        .into_response())
}
