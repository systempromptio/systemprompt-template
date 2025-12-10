use super::{TokenError, TokenErrorResponse, TokenResponse, TokenResult};
use crate::repository::OAuthRepository;
use crate::services::{generate_jwt, JwtConfig};
use anyhow::Result;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use systemprompt_core_logging::LogLevel;
use systemprompt_identifiers::SessionId;
use systemprompt_models::auth::{parse_permissions, AuthenticatedUser, Permission};

pub async fn generate_tokens_by_user_id(
    repo: &OAuthRepository,
    client_id: &str,
    user_id: &str,
    scope: Option<&str>,
    headers: &HeaderMap,
    ctx: &systemprompt_core_system::AppContext,
) -> Result<TokenResponse> {
    use crate::services::{generate_access_token_jti, generate_secure_token};

    let expires_in = systemprompt_core_system::Config::global().jwt_access_token_expiration;
    let refresh_token_expires_in =
        systemprompt_core_system::Config::global().jwt_refresh_token_expiration;

    let scope_str =
        scope.ok_or_else(|| anyhow::anyhow!("Scope is required for token generation"))?;

    let user = load_authenticated_user(repo, user_id).await?;

    let requested_permissions = parse_permissions(scope_str)?;
    let user_perms = user.permissions().to_vec();
    let final_permissions =
        resolve_user_permissions(repo, &requested_permissions, &user_perms, client_id).await?;
    let scope_string = systemprompt_models::auth::permissions_to_string(&final_permissions);

    let user_id_typed = systemprompt_identifiers::UserId::new(user_id.to_string());
    let session_service = crate::services::SessionCreationService::new(
        ctx.analytics_service().clone(),
        systemprompt_core_users::repository::UserRepository::new(ctx.db_pool().clone()),
    );
    let session_id = session_service
        .create_authenticated_session(&user_id_typed, headers)
        .await?;

    let access_token_jti = generate_access_token_jti();
    let config = JwtConfig {
        permissions: final_permissions,
        ..Default::default()
    };
    let jwt_secret = &systemprompt_core_system::Config::global().jwt_secret;
    let access_token = generate_jwt(&user, config, access_token_jti, &session_id, jwt_secret)?;

    let refresh_token_value = generate_secure_token("rt");
    let refresh_expires_at = chrono::Utc::now().timestamp() + refresh_token_expires_in;

    repo.store_refresh_token(
        &refresh_token_value,
        client_id,
        user_id,
        &scope_string,
        refresh_expires_at,
    )
    .await?;

    let _ = repo.update_client_last_used(client_id).await;

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        refresh_token: Some(refresh_token_value),
        scope: Some(scope_string),
    })
}

pub async fn load_authenticated_user(
    repo: &OAuthRepository,
    user_id: &str,
) -> Result<AuthenticatedUser> {
    repo.get_authenticated_user(user_id).await
}

pub async fn generate_client_tokens(
    _repo: &OAuthRepository,
    client_id: &str,
    scope: Option<&str>,
) -> Result<TokenResponse> {
    let expires_in = systemprompt_core_system::Config::global().jwt_access_token_expiration;

    let scope_str =
        scope.ok_or_else(|| anyhow::anyhow!("Scope is required for client credentials grant"))?;

    let permissions = parse_permissions(scope_str)?;

    let mut hasher = Sha256::new();
    hasher.update(format!("client.{client_id}").as_bytes());
    let hash = hasher.finalize();

    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(&hash[..16]);
    let client_uuid = uuid::Uuid::from_bytes(uuid_bytes);

    let client_user = AuthenticatedUser::new(
        client_uuid,
        format!("client:{client_id}"),
        None,
        vec![Permission::Admin],
    );

    let config = JwtConfig {
        permissions: permissions.clone(),
        ..Default::default()
    };
    let jwt_secret = &systemprompt_core_system::Config::global().jwt_secret;
    let session_id = SessionId::new(format!("sess_{}", uuid::Uuid::new_v4().simple()));
    let jwt_token = generate_jwt(
        &client_user,
        config,
        uuid::Uuid::new_v4().to_string(),
        &session_id,
        jwt_secret,
    )?;

    Ok(TokenResponse {
        access_token: jwt_token,
        token_type: "Bearer".to_string(),
        expires_in,
        refresh_token: None,
        scope: Some(
            permissions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" "),
        ),
    })
}

pub async fn resolve_user_permissions(
    repo: &OAuthRepository,
    requested_permissions: &[Permission],
    user_permissions: &[Permission],
    client_id: &str,
) -> Result<Vec<Permission>> {
    let client = repo
        .find_client(client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

    let client_allowed: Vec<Permission> = client
        .scopes
        .iter()
        .filter_map(|s| Permission::from_str(s).ok())
        .collect();

    let mut final_permissions = Vec::new();

    for requested in requested_permissions {
        if !client_allowed.contains(requested) {
            continue;
        }

        if *requested == Permission::User {
            final_permissions.extend(
                user_permissions
                    .iter()
                    .filter(|p| p.is_user_role())
                    .copied(),
            );
        } else if user_permissions.contains(requested) {
            final_permissions.push(*requested);
        }
    }

    final_permissions.sort_by_key(|p| std::cmp::Reverse(p.hierarchy_level()));
    final_permissions.dedup();

    if final_permissions.is_empty() {
        return Err(anyhow::anyhow!("No valid permissions available for user"));
    }

    Ok(final_permissions)
}

pub fn convert_token_result_to_response(
    result: TokenResult<TokenResponse>,
) -> axum::response::Response {
    match result {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => {
            let status = match &error {
                TokenError::InvalidClientSecret => StatusCode::UNAUTHORIZED,
                TokenError::ServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            };
            let error_response: TokenErrorResponse = error.into();
            (status, Json(error_response)).into_response()
        },
    }
}

pub async fn log_token_success(
    logger: &systemprompt_core_logging::LogService,
    grant_type: &str,
    client_id: Option<&str>,
    user_id: Option<&str>,
    granted_scopes: &str,
    token_response: &TokenResponse,
) {
    logger
        .log(
            LogLevel::Info,
            "oauth_token",
            "Access token generated",
            Some(serde_json::json!({
                "client_id": client_id,
                "grant_type": grant_type,
                "user_id": user_id,
                "granted_scopes": granted_scopes,
                "token_expires_at": chrono::Utc::now().timestamp() + token_response.expires_in,
                "refresh_token_issued": token_response.refresh_token.is_some(),
                "token_hash": hash_token(&token_response.access_token)
            })),
        )
        .await
        .ok();
}

pub async fn log_token_error(
    logger: &systemprompt_core_logging::LogService,
    error: &TokenError,
    grant_type: &str,
    client_id: Option<&str>,
) {
    let (denial_reason, detailed_message) = match error {
        TokenError::InvalidClientSecret => ("invalid_client_secret", error.to_string()),
        TokenError::InvalidGrant { ref reason } => (
            "invalid_authorization_code",
            format!("OAuth validation failed: {reason}"),
        ),
        TokenError::ServerError { ref message } => {
            ("server_error", format!("OAuth server error: {message}"))
        },
        _ => ("unknown_error", error.to_string()),
    };

    logger
        .log(
            LogLevel::Error,
            "oauth_token",
            &detailed_message,
            Some(serde_json::json!({
                "client_id": client_id,
                "grant_type": grant_type,
                "denial_reason": denial_reason,
                "error_code": match error {
                    TokenError::InvalidClientSecret => "invalid_client",
                    TokenError::InvalidGrant { .. } => "invalid_grant",
                    TokenError::ServerError { .. } => "server_error",
                    _ => "unknown_error",
                },
                "error_details": error.to_string()
            })),
        )
        .await
        .ok();
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
