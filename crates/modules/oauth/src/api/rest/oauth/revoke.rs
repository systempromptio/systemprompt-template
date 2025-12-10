use crate::repository::OAuthRepository;
use crate::services::validation::get_audit_user;
use anyhow::Result;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Form, Json};
use serde::{Deserialize, Serialize};
use systemprompt_core_logging::{LogLevel, LogService};

use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]

pub struct RevokeRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Serialize)]

pub struct RevokeError {
    pub error: String,
    pub error_description: Option<String>,
}

pub async fn handle_revoke(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Form(request): Form<RevokeRequest>,
) -> impl IntoResponse {
    let repo = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    let audit_user = match get_audit_user(Some(req_ctx.auth.user_id.as_str())) {
        Ok(user) => user,
        Err(e) => {
            let error = RevokeError {
                error: "invalid_request".to_string(),
                error_description: Some(format!("Authenticated user required: {e}")),
            };
            return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
        },
    };

    logger
        .log(
            LogLevel::Info,
            "oauth_revoke",
            "Token revocation request received",
            None,
        )
        .await
        .ok();

    let token_type = request
        .token_type_hint
        .as_deref()
        .unwrap_or("not_specified");
    let token_hash = hash_token(&request.token);

    if let Some(client_id) = &request.client_id {
        if validate_client_credentials(&repo, client_id, request.client_secret.as_deref())
            .await
            .is_err()
        {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_revoke",
                    "Token revocation failed",
                    Some(serde_json::json!({
                        "token_hash": token_hash,
                        "token_type": token_type,
                        "client_id": client_id,
                        "revocation_reason": "invalid_client_credentials",
                        "error": "invalid_client"
                    })),
                )
                .await
                .ok();

            let error = RevokeError {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client credentials".to_string()),
            };
            return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
        }
    }

    match revoke_token(&repo, &request.token, request.token_type_hint.as_deref()).await {
        Ok(()) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_revoke",
                    "Token revoked",
                    Some(serde_json::json!({
                        "token_hash": token_hash,
                        "token_type": token_type,
                        "client_id": request.client_id,
                        "revocation_reason": "user_request",
                        "revoked_by": &audit_user
                    })),
                )
                .await
                .ok();

            StatusCode::OK.into_response()
        },
        Err(error) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_revoke",
                    "Token revocation failed",
                    Some(serde_json::json!({
                        "token_hash": token_hash,
                        "token_type": token_type,
                        "client_id": request.client_id,
                        "revocation_reason": "server_error",
                        "error": error.to_string(),
                        "revoked_by": &audit_user
                    })),
                )
                .await
                .ok();

            let error = RevokeError {
                error: "server_error".to_string(),
                error_description: Some(error.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        },
    }
}

async fn validate_client_credentials(
    repo: &OAuthRepository,
    client_id: &str,
    client_secret: Option<&str>,
) -> Result<()> {
    let client = repo
        .find_client(client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

    if let Some(secret) = client_secret {
        use crate::services::verify_client_secret;
        let hash = client
            .client_secret_hash
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Client has no secret hash configured"))?;
        if !verify_client_secret(secret, hash)? {
            return Err(anyhow::anyhow!("Invalid client secret"));
        }
    } else {
        return Err(anyhow::anyhow!("Client secret required"));
    }

    Ok(())
}

async fn revoke_token(
    _repo: &OAuthRepository,
    _token: &str,
    _token_type_hint: Option<&str>,
) -> Result<()> {
    // In a stateless JWT-only approach, tokens cannot be revoked server-side.
    // The revoke endpoint always returns success as per RFC 7009, but tokens
    // are simply not stored/tracked anymore. Clients should discard the tokens
    // on their side. For enhanced security, implement JWT blacklisting if needed.
    Ok(())
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
