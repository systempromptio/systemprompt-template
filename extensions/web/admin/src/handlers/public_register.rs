//! Public self-registration and the one-shot setup token it issues.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::types::CreateUserRequest;

const TOKEN_PREFIX: &str = "sp_wst_";

#[derive(Deserialize, Debug)]
pub(crate) struct PublicRegisterRequest {
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PublicRegisterResponse {
    pub ok: bool,
    pub token: String,
    pub email: String,
    pub user_id: UserId,
    pub display_name: String,
}

pub(crate) async fn public_register_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<PublicRegisterRequest>,
) -> AdminResult<Response> {
    let email_str = body.email.trim().to_lowercase();
    let name = body.name.trim().to_owned();

    validate_registration_input(&email_str, &name)?;

    let email = Email::try_new(email_str.clone())
        .map_err(|e| AdminError::BadRequest(format!("Invalid email address: {e}")))?;

    check_rate_limit(&pool, &email_str).await?;

    let user = create_registration_user(&pool, &name, email, &body.role).await?;

    let (raw_token, token_hash) = generate_setup_token();
    let token_id = uuid::Uuid::new_v4().to_string();

    repositories::users::registration::insert_setup_token(
        pool.as_ref(),
        &token_id,
        &user.user_id,
        &token_hash,
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(PublicRegisterResponse {
            ok: true,
            token: raw_token,
            email: email_str,
            user_id: user.user_id.clone(),
            display_name: name,
        }),
    )
        .into_response())
}

fn validate_registration_input(email_str: &str, name: &str) -> AdminResult<()> {
    if email_str.is_empty() || !email_str.contains('@') {
        return Err(AdminError::BadRequest("Invalid email address".to_owned()));
    }
    if name.is_empty() {
        return Err(AdminError::BadRequest("Name is required".to_owned()));
    }
    Ok(())
}

async fn check_rate_limit(pool: &PgPool, email_str: &str) -> AdminResult<()> {
    let rate_count =
        repositories::users::registration::count_recent_setup_tokens(pool, email_str).await;

    if rate_count >= 5 {
        return Err(AdminError::RateLimited(
            "Too many registration attempts. Please try again later.".to_owned(),
        ));
    }
    Ok(())
}

async fn create_registration_user(
    pool: &PgPool,
    name: &str,
    email: Email,
    role: &str,
) -> AdminResult<crate::types::UserSummary> {
    let user_id = UserId::new(uuid::Uuid::new_v4().to_string());
    let roles = match role {
        "admin" => vec!["user".to_owned(), "admin".to_owned()],
        _ => vec!["user".to_owned()],
    };

    let create_req = CreateUserRequest {
        user_id,
        display_name: name.to_owned(),
        email,
        roles,
        status: Some("active".to_owned()),
    };

    Ok(repositories::users::mutations::create_user(pool, &create_req).await?)
}

fn generate_setup_token() -> (String, String) {
    let bytes: [u8; 32] = rand::rng().random();
    let raw_token = format!("{}{}", TOKEN_PREFIX, URL_SAFE_NO_PAD.encode(bytes));
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    };
    (raw_token, token_hash)
}
