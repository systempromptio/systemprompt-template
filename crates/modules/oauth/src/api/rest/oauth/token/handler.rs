use super::generation::{
    convert_token_result_to_response, generate_client_tokens, generate_tokens_by_user_id,
    log_token_error, log_token_success,
};
use super::validation::{
    extract_required_field, validate_authorization_code, validate_client_credentials,
};
use super::{TokenError, TokenRequest};
use crate::repository::OAuthRepository;
use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Form;
use systemprompt_core_logging::{LogLevel, LogService};

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

        let client_id = if let Some(id) = request.client_id.as_deref() {
            id.to_string()
        } else {
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
        let refresh_token =
            extract_required_field(request.refresh_token.as_deref(), "refresh_token")?;

        validate_client_credentials(&repo, client_id, request.client_secret.as_deref())
            .await
            .map_err(|_| TokenError::InvalidClientSecret)?;

        let (user_id, original_scope) = repo
            .consume_refresh_token(refresh_token, client_id)
            .await
            .map_err(|e| TokenError::InvalidRefreshToken {
                reason: e.to_string(),
            })?;

        let effective_scope = if let Some(requested_scope) = request.scope.as_deref() {
            let original_scopes = OAuthRepository::parse_scopes(&original_scope);
            let requested_scopes = OAuthRepository::parse_scopes(requested_scope);

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
        log_token_error(logger, error, "refresh_token", request.client_id.as_deref()).await;
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
