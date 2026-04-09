use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use systemprompt::identifiers::Email;

use crate::admin::repositories::magic_links;

#[derive(Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

pub async fn request_magic_link(
    State(pool): State<Arc<PgPool>>,
    req_headers: HeaderMap,
    Json(body): Json<MagicLinkRequest>,
) -> impl IntoResponse {
    let email = body.email.trim().to_lowercase();

    if Email::try_new(email.clone()).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid email address"})),
        );
    }

    let count = magic_links::count_recent_tokens(&pool, &email)
        .await
        .unwrap_or(0);

    if count >= 3 {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "ok": true,
                "message": "If an account exists for that email, a magic link has been sent."
            })),
        );
    }

    let ip_address = req_headers
        .get("x-forwarded-for")
        .or_else(|| req_headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map_or_else(
            || "unknown".to_string(),
            |s| s.split(',').next().unwrap_or(s).trim().to_string(),
        );

    let ip_count = magic_links::count_recent_tokens_by_ip(&pool, &ip_address)
        .await
        .unwrap_or(0);

    if ip_count >= 10 {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "ok": true,
                "message": "If an account exists for that email, a magic link has been sent."
            })),
        );
    }

    let user_exists = magic_links::user_exists_by_email(&pool, &email)
        .await
        .unwrap_or(false);

    if user_exists {
        if let Ok(_raw_token) =
            magic_links::create_magic_link_token(&pool, &email, Some(&ip_address)).await
        {
            tracing::info!(email = %email, "Magic link token created (email sending not configured in this deployment)");
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "message": "If an account exists for that email, a magic link has been sent."
        })),
    )
}

#[derive(Deserialize)]
pub struct ValidateTokenRequest {
    pub token: String,
}

pub async fn validate_magic_link(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<ValidateTokenRequest>,
) -> impl IntoResponse {
    match magic_links::consume_magic_link_token(&pool, &body.token).await {
        Ok(email) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "ok": true,
                "email": email
            })),
        ),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": "This link is invalid or has expired. Please request a new one."
            })),
        ),
    }
}
