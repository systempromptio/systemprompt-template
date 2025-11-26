use crate::repository::OAuthRepository;
use crate::services::{generate_jwt, JwtConfig};
use anyhow::Result;
use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Form, Json,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_identifiers::SessionId;
use systemprompt_models::auth::{parse_permissions, AuthenticatedUser, Permission};

use sha2::{Digest, Sha256};

type TokenResult<T> = Result<T, TokenError>;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub code_verifier: Option<String>,
}

#[derive(Debug, Serialize)]

pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Invalid request: {field} {message}")]
    InvalidRequest { field: String, message: String },

    #[error("Unsupported grant type: {grant_type}")]
    UnsupportedGrantType { grant_type: String },

    #[error("Invalid client credentials")]
    InvalidClient,

    #[error("Invalid authorization code: {reason}")]
    InvalidGrant { reason: String },

    #[error("Invalid refresh token: {reason}")]
    InvalidRefreshToken { reason: String },

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid client secret")]
    InvalidClientSecret,

    #[error("Authorization code expired")]
    ExpiredCode,

    #[error("Server error: {message}")]
    ServerError { message: String },
}

#[derive(Debug, Serialize)]

pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

impl From<TokenError> for TokenErrorResponse {
    fn from(error: TokenError) -> Self {
        let (error_type, description) = match &error {
            TokenError::InvalidRequest { field, message } => {
                ("invalid_request", Some(format!("{field}: {message}")))
            },
            TokenError::UnsupportedGrantType { grant_type } => (
                "unsupported_grant_type",
                Some(format!("Grant type '{grant_type}' is not supported")),
            ),
            TokenError::InvalidClient => (
                "invalid_client",
                Some("Client authentication failed".to_string()),
            ),
            TokenError::InvalidGrant { reason } => ("invalid_grant", Some(reason.clone())),
            TokenError::InvalidRefreshToken { reason } => (
                "invalid_grant",
                Some(format!("Refresh token invalid: {reason}")),
            ),
            TokenError::InvalidCredentials => {
                ("invalid_grant", Some("Invalid credentials".to_string()))
            },
            TokenError::InvalidClientSecret => {
                ("invalid_client", Some("Invalid client secret".to_string()))
            },
            TokenError::ExpiredCode => (
                "invalid_grant",
                Some("Authorization code expired".to_string()),
            ),
            TokenError::ServerError { message } => ("server_error", Some(message.clone())),
        };

        Self {
            error: error_type.to_string(),
            error_description: description,
        }
    }
}

pub async fn handle_token(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    headers: HeaderMap,
    Form(request): Form<TokenRequest>,
) -> impl IntoResponse {
    let repo = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    logger
        .log(
            LogLevel::Info,
            "oauth_token",
            &format!(
                "Token request received for grant_type: {}",
                request.grant_type
            ),
            None,
        )
        .await
        .ok();

    let response = match request.grant_type.as_str() {
        "authorization_code" => {
            handle_authorization_code_grant(repo, request, &logger, &headers, &ctx).await
        },
        "refresh_token" => handle_refresh_token_grant(repo, request, &logger, &headers, &ctx).await,
        "client_credentials" => handle_client_credentials_grant(repo, request, &logger).await,
        _ => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_token",
                    "Token request denied",
                    Some(serde_json::json!({
                        "client_id": request.client_id,
                        "grant_type": request.grant_type,
                        "denial_reason": "unsupported_grant_type",
                        "error_code": "unsupported_grant_type"
                    })),
                )
                .await
                .ok();

            let error = TokenError::UnsupportedGrantType {
                grant_type: request.grant_type.clone(),
            };
            convert_token_result_to_response(Err(error))
        },
    };

    response
}

async fn handle_authorization_code_grant(
    repo: OAuthRepository,
    request: TokenRequest,
    logger: &LogService,
    headers: &HeaderMap,
    ctx: &systemprompt_core_system::AppContext,
) -> axum::response::Response {
    let result = async {
        let code = extract_required_field(request.code.as_deref(), "code")?;

        // Try explicit client_id first, fallback to inferring from code for MCP client compatibility
        let client_id = if let Some(id) = request.client_id.as_deref() {
            id.to_string()
        } else {
            // Compatibility mode: infer client_id from authorization code
            // Required for MCP clients that don't send client_id in token exchange
            repo.get_client_id_from_auth_code(code)
                .await
                .map_err(|e| TokenError::ServerError {
                    message: format!("Failed to lookup authorization code: {e}"),
                })?
                .ok_or_else(|| TokenError::InvalidGrant {
                    reason: "Invalid or expired authorization code".to_string(),
                })?
        };

        validate_client_credentials(&repo, &client_id, request.client_secret.as_deref())
            .await
            .map_err(|_| TokenError::InvalidClientSecret)?;

        let (user_id, authorized_scope) = validate_authorization_code(
            &repo,
            code,
            &client_id,
            request.redirect_uri.as_deref(),
            request.code_verifier.as_deref(),
        )
        .await
        .map_err(|e| TokenError::InvalidGrant {
            reason: e.to_string(),
        })?;

        // Use the scope from the authorization code, not the request scope
        let token_response = generate_tokens_by_user_id(
            &repo,
            &client_id,
            &user_id,
            Some(&authorized_scope),
            headers,
            ctx,
        )
        .await
        .map_err(|e| TokenError::ServerError {
            message: e.to_string(),
        })?;

        log_token_success(
            logger,
            "authorization_code",
            Some(&client_id),
            Some(&user_id),
            &authorized_scope,
            &token_response,
        )
        .await;

        Ok(token_response)
    }
    .await;

    if let Err(ref error) = result {
        log_token_error(
            logger,
            error,
            "authorization_code",
            request.client_id.as_deref(),
        )
        .await;
    }

    convert_token_result_to_response(result)
}

async fn handle_refresh_token_grant(
    repo: OAuthRepository,
    request: TokenRequest,
    logger: &LogService,
    headers: &HeaderMap,
    ctx: &systemprompt_core_system::AppContext,
) -> axum::response::Response {
    let result = async {
        let client_id = extract_required_field(request.client_id.as_deref(), "client_id")?;
        let refresh_token = extract_required_field(request.refresh_token.as_deref(), "refresh_token")?;

        validate_client_credentials(&repo, client_id, request.client_secret.as_deref())
            .await
            .map_err(|_| TokenError::InvalidClientSecret)?;

        let (user_id, original_scope) = repo
            .consume_refresh_token(refresh_token, client_id)
            .await
            .map_err(|e| TokenError::InvalidRefreshToken {
                reason: e.to_string(),
            })?;

        // Validate that requested scope is subset of original scope
        let effective_scope = if let Some(requested_scope) = request.scope.as_deref() {
            let original_scopes = OAuthRepository::parse_scopes(&original_scope);
            let requested_scopes = OAuthRepository::parse_scopes(requested_scope);

            // Check that all requested scopes are in the original scope
            for requested in &requested_scopes {
                if !original_scopes.contains(requested) {
                    return Err(TokenError::InvalidRequest {
                        field: "scope".to_string(),
                        message: format!("Requested scope '{requested}' not in original scope"),
                    });
                }
            }
            requested_scope
        } else {
            &original_scope
        };

        let token_response = generate_tokens_by_user_id(
            &repo,
            client_id,
            &user_id,
            Some(effective_scope),
            headers,
            ctx,
        )
        .await
        .map_err(|e| TokenError::ServerError {
            message: e.to_string(),
        })?;

        log_token_success(
            logger,
            "refresh_token",
            Some(client_id),
            Some(&user_id),
            effective_scope,
            &token_response,
        )
        .await;

        Ok(token_response)
    }
    .await;

    if let Err(ref error) = result {
        log_token_error(
            logger,
            error,
            "refresh_token",
            request.client_id.as_deref(),
        )
        .await;
    }

    convert_token_result_to_response(result)
}

async fn handle_client_credentials_grant(
    repo: OAuthRepository,
    request: TokenRequest,
    logger: &LogService,
) -> axum::response::Response {
    let result = async {
        let client_id = extract_required_field(request.client_id.as_deref(), "client_id")?;

        validate_client_credentials(&repo, client_id, request.client_secret.as_deref())
            .await
            .map_err(|_| TokenError::InvalidClientSecret)?;

        let token_response = generate_client_tokens(&repo, client_id, request.scope.as_deref())
            .await
            .map_err(|e| TokenError::ServerError {
                message: e.to_string(),
            })?;

        log_token_success(
            logger,
            "client_credentials",
            Some(client_id),
            None,
            token_response.scope.as_deref().unwrap_or(""),
            &token_response,
        )
        .await;

        Ok(token_response)
    }
    .await;

    if let Err(ref error) = result {
        log_token_error(
            logger,
            error,
            "client_credentials",
            request.client_id.as_deref(),
        )
        .await;
    }

    convert_token_result_to_response(result)
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

    // Check authentication method
    let auth_method = client.token_endpoint_auth_method.as_str();

    match auth_method {
        "none" => {
            // No authentication required
            Ok(())
        },
        _ => {
            // Require client secret for other authentication methods
            if let Some(secret) = client_secret {
                use crate::services::verify_client_secret;
                let hash = client
                    .client_secret_hash
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Client has no secret hash configured"))?;
                if !verify_client_secret(secret, hash)? {
                    return Err(anyhow::anyhow!("Invalid client secret"));
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Client secret required"))
            }
        },
    }
}

async fn validate_authorization_code(
    repo: &OAuthRepository,
    code: &str,
    client_id: &str,
    redirect_uri: Option<&str>,
    code_verifier: Option<&str>,
) -> Result<(String, String)> {
    let (user_id, scope) = repo
        .validate_authorization_code(code, client_id, redirect_uri, code_verifier)
        .await?;
    Ok((user_id, scope))
}

async fn generate_tokens_by_user_id(
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

    // Parse and validate scopes - scope is required
    let scope_str =
        scope.ok_or_else(|| anyhow::anyhow!("Scope is required for token generation"))?;

    // Load user from database by ID to get their actual permissions
    let user = load_authenticated_user(repo, user_id).await?;

    // Resolve final permissions by intersecting requested scopes with user's actual permissions
    let requested_permissions = parse_permissions(scope_str)?;
    let user_perms = user.permissions().to_vec();
    let final_permissions =
        resolve_user_permissions(repo, &requested_permissions, &user_perms, client_id).await?;
    let scope_string = systemprompt_models::auth::permissions_to_string(&final_permissions);

    // Create session for authenticated user with analytics
    let user_id_typed = systemprompt_identifiers::UserId::new(user_id.to_string());
    let session_service = crate::services::SessionCreationService::new(
        ctx.analytics_service().clone(),
        systemprompt_core_users::repository::UserRepository::new(ctx.db_pool().clone()),
    );
    let session_id = session_service
        .create_authenticated_session(&user_id_typed, headers)
        .await?;

    // Generate access token
    let access_token_jti = generate_access_token_jti();
    let config = JwtConfig {
        permissions: final_permissions,
        ..Default::default()
    };
    let jwt_secret = &systemprompt_core_system::Config::global().jwt_secret;
    let access_token = generate_jwt(&user, config, access_token_jti, &session_id, jwt_secret)?;

    // Generate and store refresh token
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

    // Track client usage (ignore errors to avoid blocking token generation)
    let _ = repo.update_client_last_used(client_id).await;

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        refresh_token: Some(refresh_token_value),
        scope: Some(scope_string),
    })
}

async fn load_authenticated_user(
    repo: &OAuthRepository,
    user_id: &str,
) -> Result<AuthenticatedUser> {
    repo.get_authenticated_user(user_id).await
}

fn extract_required_field<'a>(field: Option<&'a str>, field_name: &str) -> TokenResult<&'a str> {
    field.ok_or_else(|| TokenError::InvalidRequest {
        field: field_name.to_string(),
        message: "is required".to_string(),
    })
}

fn convert_token_result_to_response(
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

async fn generate_client_tokens(
    _repo: &OAuthRepository,
    client_id: &str,
    scope: Option<&str>,
) -> Result<TokenResponse> {
    let expires_in = systemprompt_core_system::Config::global().jwt_access_token_expiration;

    let scope_str =
        scope.ok_or_else(|| anyhow::anyhow!("Scope is required for client credentials grant"))?;

    let permissions = parse_permissions(scope_str)?;

    // Generate deterministic UUID for client to ensure consistency across token generations
    // Use a hash-based approach since v5 UUID might not be available
    let mut hasher = Sha256::new();
    hasher.update(format!("client.{client_id}").as_bytes());
    let hash = hasher.finalize();

    // Take first 16 bytes of hash for UUID
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

async fn resolve_user_permissions(
    repo: &OAuthRepository,
    requested_permissions: &[systemprompt_models::auth::Permission],
    user_permissions: &[systemprompt_models::auth::Permission],
    client_id: &str,
) -> Result<Vec<Permission>> {
    // Get client's allowed scopes
    let client = repo
        .find_client(client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

    // Parse client's allowed scopes from database (stored as strings)
    let client_allowed: Vec<Permission> = client
        .scopes
        .iter()
        .filter_map(|s| Permission::from_str(s).ok())
        .collect();

    let mut final_permissions = Vec::new();

    // For each requested permission, check if user actually has it
    for requested in requested_permissions {
        // Check if client is allowed this permission
        if !client_allowed.contains(requested) {
            continue; // Skip permissions client isn't allowed
        }

        // Special handling for "user" permission - it auto-upgrades to user's actual user-level permissions
        if *requested == Permission::User {
            // Include all user-level permissions the user actually has
            final_permissions.extend(
                user_permissions
                    .iter()
                    .filter(|p| p.is_user_role())
                    .copied(),
            );
        } else {
            // For other permissions, user must explicitly have that permission
            if user_permissions.contains(requested) {
                final_permissions.push(*requested);
            }
        }
    }

    // Sort by hierarchy level for consistency
    final_permissions.sort_by_key(|p| std::cmp::Reverse(p.hierarchy_level()));
    final_permissions.dedup();

    if final_permissions.is_empty() {
        return Err(anyhow::anyhow!("No valid permissions available for user"));
    }

    Ok(final_permissions)
}

async fn log_token_success(
    logger: &LogService,
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

async fn log_token_error(
    logger: &LogService,
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

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
