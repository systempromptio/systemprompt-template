use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use crate::repositories;
use crate::types::CreateUserRequest;

use super::shared;

const TOKEN_PREFIX: &str = "sp_wst_";

#[derive(Deserialize, Debug)]
pub struct PublicRegisterRequest {
    pub name: String,
    pub email: String,
    pub role: String,
}

pub async fn public_register_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<PublicRegisterRequest>,
) -> impl IntoResponse {
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

    let rate_count =
        repositories::registration::count_recent_setup_tokens(pool.as_ref(), &email_str).await;

    if rate_count >= 5 {
        return shared::error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many registration attempts. Please try again later.",
        );
    }

    let user_id = UserId::new(uuid::Uuid::new_v4().to_string());

    let roles = match body.role.as_str() {
        "admin" => vec!["user".to_string(), "admin".to_string()],
        _ => vec!["user".to_string()],
    };

    let create_req = CreateUserRequest {
        user_id: user_id.clone(),
        display_name: name.clone(),
        email,
        roles,
        status: Some("active".to_string()),
    };

    let user = match repositories::create_user(&pool, &create_req).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user during public registration");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to create user: {e}"),
            );
        }
    };

    let bytes: [u8; 32] = rand::rng().random();
    let raw_token = format!("{}{}", TOKEN_PREFIX, URL_SAFE_NO_PAD.encode(bytes));
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    };

    let token_id = uuid::Uuid::new_v4().to_string();

    if let Err(e) = repositories::registration::insert_setup_token(
        pool.as_ref(),
        &token_id,
        user.user_id.as_str(),
        &token_hash,
    )
    .await
    {
        tracing::error!(error = %e, "Failed to create setup token");
        return shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "User created but failed to generate setup token",
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "token": raw_token,
            "email": email_str,
            "user_id": user.user_id,
            "display_name": name,
        })),
    )
        .into_response()
}
