use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use crate::repositories;
use crate::repositories::magic_links;
use crate::types::{CreateUserRequest, UserContext};

use super::shared;

#[derive(Deserialize, Debug)]
pub struct DemoRegisterRequest {
    pub name: String,
    pub email: String,
    pub role: String,
}

pub async fn create_demo_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<DemoRegisterRequest>,
) -> impl IntoResponse {
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }

    let email_str = body.email.trim().to_lowercase();
    let name = body.name.trim().to_string();

    if email_str.is_empty() || !email_str.contains('@') {
        return shared::error_response(StatusCode::BAD_REQUEST, "Invalid email address");
    }
    if name.is_empty() {
        return shared::error_response(StatusCode::BAD_REQUEST, "Name is required");
    }

    let Ok(email) = Email::try_new(email_str.clone()) else {
        return shared::error_response(StatusCode::BAD_REQUEST, "Invalid email address");
    };

    let local_part = email_str.split('@').next().unwrap_or("user");
    let sanitized = local_part
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .trim_matches('-')
        .to_string();
    let user_id = UserId::new(if sanitized.is_empty() {
        "user".to_string()
    } else {
        sanitized
    });

    let roles = match body.role.as_str() {
        "admin" => vec!["admin".to_string()],
        _ => vec![],
    };

    let create_req = CreateUserRequest {
        user_id: user_id.clone(),
        display_name: name.clone(),
        email,
        roles,
        status: Some("active".to_string()),
    };

    if let Err(e) = repositories::create_user(&pool, &create_req).await {
        tracing::error!(error = %e, "Failed to create demo user");
        return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Registration failed");
    }

    let raw_token = match magic_links::create_magic_link_token(&pool, &email_str, None).await {
        Ok(token) => token,
        Err(e) => {
            tracing::error!(error = %e, "Failed to create registration token");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "User created but failed to generate registration link",
            );
        }
    };

    let registration_url = format!("/admin/add-passkey?token={raw_token}");

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "registration_url": registration_url,
            "user_id": user_id.as_str(),
            "display_name": name,
        })),
    )
        .into_response()
}
