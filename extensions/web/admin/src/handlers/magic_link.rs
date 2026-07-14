use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use systemprompt::identifiers::Email;

use crate::repositories::magic_links;

use super::shared::ErrorBody;

#[derive(Deserialize, Debug)]
pub(crate) struct MagicLinkRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct MagicLinkResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum MagicLinkRequestResult {
    Ok(MagicLinkResponse),
    Err(ErrorBody),
}

const RATE_LIMITED_MESSAGE: &str =
    "If an account exists for that email, a magic link has been sent.";

pub(crate) async fn request_magic_link(
    State(pool): State<Arc<PgPool>>,
    req_headers: HeaderMap,
    Json(body): Json<MagicLinkRequest>,
) -> impl IntoResponse {
    let email = body.email.trim().to_lowercase();

    if Email::try_new(email.clone()).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MagicLinkRequestResult::Err(ErrorBody {
                error: "Invalid email address".to_owned(),
            })),
        );
    }

    let count = magic_links::count_recent_tokens(&pool, &email)
        .await
        .unwrap_or(0);

    if count >= 3 {
        return (
            StatusCode::OK,
            Json(MagicLinkRequestResult::Ok(MagicLinkResponse {
                ok: true,
                message: RATE_LIMITED_MESSAGE.to_owned(),
            })),
        );
    }

    let ip_address = req_headers
        .get("x-forwarded-for")
        .or_else(|| req_headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map_or_else(
            || "unknown".to_owned(),
            |s| s.split(',').next().unwrap_or(s).trim().to_owned(),
        );

    let ip_count = magic_links::count_recent_tokens_by_ip(&pool, &ip_address)
        .await
        .unwrap_or(0);

    if ip_count >= 10 {
        return (
            StatusCode::OK,
            Json(MagicLinkRequestResult::Ok(MagicLinkResponse {
                ok: true,
                message: RATE_LIMITED_MESSAGE.to_owned(),
            })),
        );
    }

    let user_exists = magic_links::user_exists_by_email(&pool, &email)
        .await
        .unwrap_or(false);

    if user_exists
        && let Ok(_raw_token) =
            magic_links::create_magic_link_token(&pool, &email, Some(&ip_address)).await
    {
        tracing::info!(email = %email, "Magic link token created (email sending not configured in this deployment)");
    }

    (
        StatusCode::OK,
        Json(MagicLinkRequestResult::Ok(MagicLinkResponse {
            ok: true,
            message: RATE_LIMITED_MESSAGE.to_owned(),
        })),
    )
}

#[derive(Deserialize, Debug)]
pub(crate) struct ValidateTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ValidateTokenResponse {
    pub ok: bool,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ValidateTokenError {
    pub ok: bool,
    pub error: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum ValidateTokenResult {
    Ok(ValidateTokenResponse),
    Err(ValidateTokenError),
}

pub(crate) async fn validate_magic_link(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<ValidateTokenRequest>,
) -> impl IntoResponse {
    magic_links::consume_magic_link_token(&pool, &body.token)
        .await
        .map_or_else(
            |_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ValidateTokenResult::Err(ValidateTokenError {
                        ok: false,
                        error: "This link is invalid or has expired. Please request a new one."
                            .to_owned(),
                    })),
                )
            },
            |email| {
                (
                    StatusCode::OK,
                    Json(ValidateTokenResult::Ok(ValidateTokenResponse {
                        ok: true,
                        email,
                    })),
                )
            },
        )
}
