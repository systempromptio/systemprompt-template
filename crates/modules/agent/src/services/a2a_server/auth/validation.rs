use anyhow::{Context, Result};
use axum::http::{HeaderMap, StatusCode};
use std::str::FromStr;
use systemprompt_core_logging::LogService;
use systemprompt_core_oauth::services::{generate_jwt, generate_secure_token, JwtConfig};
use systemprompt_core_oauth::validate_jwt_token;
use systemprompt_core_users::repository::UserRepository;
use systemprompt_identifiers::SessionId;
use systemprompt_models::auth::{AuthenticatedUser, JwtAudience, Permission};
use uuid;

use super::types::{AgentAuthenticatedUser, AgentOAuthState};
use crate::services::a2a_server::errors::{forbidden_response, unauthorized_response};

pub async fn validate_agent_token(
    token: &str,
    state: &AgentOAuthState,
) -> Result<AgentAuthenticatedUser> {
    let log = state.log.clone();
    let claims = validate_jwt_token(
        token,
        &systemprompt_core_system::Config::global().jwt_secret,
    )
    .context("Invalid or expired JWT token")?;

    if !claims.has_audience(JwtAudience::A2a) {
        return Err(anyhow::anyhow!("Token does not support A2A protocol"));
    }

    let user_repo = UserRepository::new(state.db.clone());
    let user = verify_user_exists_and_active(&claims, &user_repo, &log).await?;

    verify_a2a_permissions(&claims, &user)?;

    let db_roles = user.roles.clone();

    log.debug(
        "a2a_auth",
        &format!(
            "Authenticated A2A user: {} ({}) - status: {}, db_roles: {:?}",
            claims.username, claims.user_type, user.status, db_roles
        ),
    )
    .await
    .ok();

    Ok(AgentAuthenticatedUser::from(claims))
}

pub async fn generate_agent_token(
    user_context: &AuthenticatedUser,
    _state: &AgentOAuthState,
) -> Result<String> {
    let jti = generate_secure_token("a2a");

    let config = JwtConfig {
        permissions: vec![Permission::A2a],
        audience: vec![JwtAudience::A2a],
        expires_in_hours: Some(1),
    };
    let session_id = SessionId::new(format!("sess_{}", uuid::Uuid::new_v4().simple()));
    generate_jwt(
        user_context,
        config,
        jti,
        &session_id,
        &systemprompt_core_system::Config::global().jwt_secret,
    )
    .context("Failed to generate A2A JWT token")
}

pub async fn generate_cross_protocol_token(
    user_context: &AuthenticatedUser,
    _state: &AgentOAuthState,
) -> Result<String> {
    let jti = generate_secure_token("cross");

    let config = JwtConfig {
        permissions: vec![Permission::Mcp, Permission::A2a],
        audience: vec![JwtAudience::Mcp, JwtAudience::A2a],
        expires_in_hours: Some(1),
    };
    let session_id = SessionId::new(format!("sess_{}", uuid::Uuid::new_v4().simple()));
    generate_jwt(
        user_context,
        config,
        jti,
        &session_id,
        &systemprompt_core_system::Config::global().jwt_secret,
    )
    .context("Failed to generate cross-protocol JWT token")
}

async fn verify_user_exists_and_active(
    claims: &systemprompt_core_oauth::JwtClaims,
    user_repo: &UserRepository,
    log: &LogService,
) -> Result<systemprompt_core_users::models::UserResponse> {
    let user = user_repo
        .get_by_id(&claims.sub)
        .await
        .context("Failed to lookup user in database")?;

    let user = match user {
        Some(u) => u,
        None => {
            log.warn(
                "a2a_auth",
                &format!("User ID {} from token not found in database", claims.sub),
            )
            .await
            .ok();
            return Err(anyhow::anyhow!("User not found"));
        },
    };

    if user.status != "active" {
        log.warn(
            "a2a_auth",
            &format!(
                "User {} has non-active status: {}",
                claims.username, user.status
            ),
        )
        .await
        .ok();
        return Err(anyhow::anyhow!("User account is not active"));
    }

    Ok(user)
}

fn verify_a2a_permissions(
    claims: &systemprompt_core_oauth::JwtClaims,
    user: &systemprompt_core_users::models::UserResponse,
) -> Result<()> {
    let token_has_a2a_permission = claims.has_permission(Permission::Admin);

    let db_permissions: Vec<Permission> = user
        .roles
        .iter()
        .filter_map(|role| Permission::from_str(role).ok())
        .collect();

    if db_permissions.is_empty() {
        return Err(anyhow::anyhow!(
            "User {} has no valid permissions",
            user.uuid
        ));
    }

    let db_has_a2a_permission = db_permissions.contains(&Permission::Admin);

    if !token_has_a2a_permission && !db_has_a2a_permission {
        return Err(anyhow::anyhow!("User lacks required A2A permissions"));
    }

    Ok(())
}

pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_header| {
            if auth_header.starts_with("Bearer ") {
                Some(auth_header[7..].to_string())
            } else {
                None
            }
        })
}

pub async fn validate_oauth_for_request(
    headers: &HeaderMap,
    request_id: &crate::models::a2a::jsonrpc::NumberOrString,
    required_scopes: &[Permission],
    log: &LogService,
) -> Result<Option<serde_json::Value>, (StatusCode, serde_json::Value)> {
    let token = match extract_bearer_token(headers) {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Err(unauthorized_response(
                "Bearer token required. Include 'Authorization: Bearer <token>' header.",
                request_id,
                log,
            )
            .await);
        },
    };

    match validate_jwt_token(
        &token,
        &systemprompt_core_system::Config::global().jwt_secret,
    ) {
        Ok(claims) => {
            log.info(
                "a2a_oauth",
                &format!("Authenticated: {} ({})", claims.username, claims.user_type),
            )
            .await
            .ok();

            if !claims.has_audience(JwtAudience::A2a) {
                return Err(forbidden_response(
                    format!(
                        "Token does not support A2A protocol. Audience: {:?}",
                        claims.aud
                    ),
                    request_id,
                    log,
                )
                .await);
            }

            if claims.is_admin() {
                log.info(
                    "a2a_oauth",
                    &format!("Admin user {} has access to all agents", claims.username),
                )
                .await
                .ok();
                return Ok(Some(serde_json::json!(claims)));
            }

            let user_permissions = claims.permissions();
            let has_required_scope = required_scopes.iter().any(|required_scope| {
                user_permissions
                    .iter()
                    .any(|user_perm| user_perm.implies(required_scope))
            });

            if !has_required_scope {
                let required_scopes_str: Vec<String> =
                    required_scopes.iter().map(|s| s.to_string()).collect();
                let user_scopes_str: Vec<String> =
                    user_permissions.iter().map(|s| s.to_string()).collect();

                log.warn(
                    "a2a_oauth",
                    &format!(
                        "Access denied: User {} lacks required scopes. Required: [{}], Has: [{}]",
                        claims.username,
                        required_scopes_str.join(", "),
                        user_scopes_str.join(", ")
                    ),
                )
                .await
                .ok();

                return Err(forbidden_response(
                    format!(
                        "User {} lacks required permissions. Required: [{}], User has: [{}]",
                        claims.username,
                        required_scopes_str.join(", "),
                        user_scopes_str.join(", ")
                    ),
                    request_id,
                    log,
                )
                .await);
            }

            Ok(Some(serde_json::json!(claims)))
        },
        Err(e) => {
            Err(
                unauthorized_response(format!("Invalid or expired token: {}", e), request_id, log)
                    .await,
            )
        },
    }
}
