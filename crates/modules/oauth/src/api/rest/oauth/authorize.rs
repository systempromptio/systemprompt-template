#![allow(unused_qualifications)]

use crate::repository::OAuthRepository;
use crate::services::validation::CsrfToken;
use crate::templates::TemplateEngine;
use anyhow::Result;
use axum::{
    extract::{Extension, Form, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use systemprompt_core_logging::{LogLevel, LogService};

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub response_mode: Option<String>,
    pub display: Option<String>,
    pub prompt: Option<String>,
    pub max_age: Option<i64>,
    pub ui_locales: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthorizeResponse {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub async fn handle_authorize_get(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    Query(params): Query<AuthorizeQuery>,
    State(ctx): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    let repo = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    logger
        .log(
            LogLevel::Info,
            "oauth_authorize",
            "Authorization request received",
            Some(serde_json::json!({
                "client_id": params.client_id,
                "response_type": params.response_type,
                "redirect_uri": params.redirect_uri,
                "requested_scopes": params.scope,
                "state_present": params.state.is_some(),
                "pkce_challenge_present": params.code_challenge.is_some(),
                "code_challenge_method": params.code_challenge_method
            })),
        )
        .await
        .ok();

    let csrf_token = match params.state.as_deref() {
        None | Some("") => {
            return (
                StatusCode::BAD_REQUEST,
                Json(AuthorizeResponse {
                    code: None,
                    state: None,
                    error: Some("invalid_request".to_string()),
                    error_description: Some("CSRF token (state parameter) is required".to_string()),
                }),
            )
                .into_response();
        },
        Some(state_str) => match CsrfToken::new(state_str) {
            Ok(token) => token,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(AuthorizeResponse {
                        code: None,
                        state: params.state.clone(),
                        error: Some("invalid_request".to_string()),
                        error_description: Some(
                            "CSRF token (state parameter) is invalid".to_string(),
                        ),
                    }),
                )
                    .into_response();
            },
        },
    };

    // Basic validation - simplified
    if params.response_type.is_empty() || params.client_id.is_empty() {
        let e = "Missing required parameters";
        if let Some(redirect_uri) = &params.redirect_uri {
            let error_url = format!(
                "{}?error=invalid_request&error_description={}&state={}",
                redirect_uri,
                urlencoding::encode(&format!("Validation error: {e}")),
                csrf_token.as_str()
            );
            return Redirect::to(&error_url).into_response();
        }
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthorizeResponse {
                code: None,
                state: params.state.clone(),
                error: Some("invalid_request".to_string()),
                error_description: Some(format!("Validation error: {e}")),
            }),
        )
            .into_response();
    }

    // Enhanced OAuth 2.0 parameter validation
    if let Err(validation_error) = validate_oauth_parameters(&params) {
        return create_error_response(
            &params,
            "invalid_request",
            &validation_error,
            StatusCode::BAD_REQUEST,
        );
    }

    match validate_authorize_request(&params, &repo).await {
        Ok(resolved_scope) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_authorize",
                    "Authorization request validated",
                    Some(serde_json::json!({
                        "client_id": params.client_id,
                        "resolved_scopes": resolved_scope,
                        "redirect_uri": params.redirect_uri,
                        "state": params.state
                    })),
                )
                .await
                .ok();

            let webauthn_form = generate_webauthn_form(&params, &resolved_scope);
            Html(webauthn_form).into_response()
        },
        Err(error) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_authorize",
                    "Authorization request denied",
                    Some(serde_json::json!({
                        "client_id": params.client_id,
                        "denial_reason": error.to_string(),
                        "requested_scopes": params.scope,
                        "redirect_uri": params.redirect_uri
                    })),
                )
                .await
                .ok();

            if let Some(redirect_uri) = &params.redirect_uri {
                let error_url = format!(
                    "{}?error=invalid_request&error_description={}&state={}",
                    redirect_uri,
                    urlencoding::encode(&error.to_string()),
                    csrf_token.as_str()
                );
                return Redirect::to(&error_url).into_response();
            }
            (
                StatusCode::BAD_REQUEST,
                Json(AuthorizeResponse {
                    code: None,
                    state: params.state,
                    error: Some("invalid_request".to_string()),
                    error_description: Some(error.to_string()),
                }),
            )
                .into_response()
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub user_consent: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub async fn handle_authorize_post(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Form(form): Form<AuthorizeRequest>,
) -> impl IntoResponse {
    let repo = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    let query = convert_form_to_query(&form);

    logger
        .log(
            LogLevel::Info,
            "oauth_authorize",
            "Authorization form submission received",
            Some(serde_json::json!({
                "client_id": form.client_id,
                "user_consent": form.user_consent,
                "username_provided": form.username.is_some(),
                "password_provided": form.password.is_some(),
                "response_type": form.response_type
            })),
        )
        .await
        .ok();

    if let Err(error) = validate_authorize_request(&query, &repo).await.map(|_| ()) {
        logger
            .log(
                LogLevel::Info,
                "oauth_authorize",
                "Authorization form validation failed",
                Some(serde_json::json!({
                    "client_id": form.client_id,
                    "validation_error": error.to_string()
                })),
            )
            .await
            .ok();
        return create_error_response(
            &query,
            "invalid_request",
            &error.to_string(),
            StatusCode::BAD_REQUEST,
        );
    }

    if !is_user_consent_granted(&form) {
        logger
            .log(
                LogLevel::Info,
                "oauth_authorize",
                "User consent denied",
                Some(serde_json::json!({
                    "client_id": form.client_id,
                    "denial_reason": "user_denied_consent",
                    "requested_scopes": form.scope
                })),
            )
            .await
            .ok();
        return create_consent_denied_response(&query);
    }

    // Since we're using WebAuthn, password-based POST authentication is not supported
    logger
        .log(
            LogLevel::Info,
            "oauth_authorize",
            "Unsupported authentication method attempted",
            Some(serde_json::json!({
                "client_id": form.client_id,
                "attempted_method": "password_based",
                "supported_method": "webauthn"
            })),
        )
        .await
        .ok();

    create_error_response(
        &query,
        "unsupported_grant_type",
        "Password authentication not supported. Use WebAuthn flow instead.",
        StatusCode::BAD_REQUEST,
    )
}

fn convert_form_to_query(form: &AuthorizeRequest) -> AuthorizeQuery {
    AuthorizeQuery {
        response_type: form.response_type.clone(),
        client_id: form.client_id.clone(),
        redirect_uri: form.redirect_uri.clone(),
        scope: form.scope.clone(),
        state: form.state.clone(),
        code_challenge: form.code_challenge.clone(),
        code_challenge_method: form.code_challenge_method.clone(),
        response_mode: None,
        display: None,
        prompt: None,
        max_age: None,
        ui_locales: None,
    }
}

fn is_user_consent_granted(form: &AuthorizeRequest) -> bool {
    form.user_consent.as_deref() == Some("allow")
}

fn create_consent_denied_response(query: &AuthorizeQuery) -> axum::response::Response {
    create_error_response(
        query,
        "access_denied",
        "User denied the request",
        StatusCode::UNAUTHORIZED,
    )
}

fn create_error_response(
    query: &AuthorizeQuery,
    error_type: &str,
    error_description: &str,
    status_code: StatusCode,
) -> axum::response::Response {
    let state_value = query.state.as_deref().unwrap_or_default();

    if let Some(redirect_uri) = &query.redirect_uri {
        let error_url = format!(
            "{}?error={}&error_description={}&state={}",
            redirect_uri,
            error_type,
            urlencoding::encode(error_description),
            state_value
        );
        return Redirect::to(&error_url).into_response();
    }

    (
        status_code,
        Json(AuthorizeResponse {
            code: None,
            state: query.state.clone(),
            error: Some(error_type.to_string()),
            error_description: Some(error_description.to_string()),
        }),
    )
        .into_response()
}

async fn validate_authorize_request(
    params: &AuthorizeQuery,
    repo: &OAuthRepository,
) -> Result<String> {
    if params.response_type != "code" {
        return Err(anyhow::anyhow!(
            "Unsupported response_type. Only 'code' is supported"
        ));
    }

    let client = repo
        .find_client(&params.client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid client_id"))?;

    if let Some(redirect_uri) = &params.redirect_uri {
        eprintln!("  Validating redirect_uri: {redirect_uri}");
        eprintln!("  Against registered URIs: {:?}", client.redirect_uris);

        let is_valid = client.redirect_uris.contains(redirect_uri);

        eprintln!("  redirect_uri validation result: {is_valid}");

        if !is_valid {
            eprintln!(
                "  ❌ redirect_uri '{redirect_uri}' not found in client registered URIs"
            );
            eprintln!("  Checking for similar URIs (potential encoding issues):");
            for registered_uri in &client.redirect_uris {
                eprintln!("    - Registered: {registered_uri}");
                if registered_uri
                    .starts_with(&redirect_uri[..redirect_uri.len().min(registered_uri.len())])
                {
                    eprintln!("      ⚠️  POTENTIAL MATCH: Similar prefix detected!");
                }
            }
            return Err(anyhow::anyhow!(
                "redirect_uri '{}' not registered for client '{}'",
                redirect_uri,
                params.client_id
            ));
        }
        eprintln!("  ✅ redirect_uri validation passed");
    }

    // Scope parameter is optional - use client's registered scopes if not provided
    let scope = if let Some(scope_param) = params.scope.as_deref() {
        scope_param.to_string()
    } else {
        // Use client's registered scopes from database
        if client.scopes.is_empty() {
            return Err(anyhow::anyhow!(
                "Client has no registered scopes and none provided in request"
            ));
        }
        client.scopes.join(" ")
    };

    let requested_scopes = crate::repository::OAuthRepository::parse_scopes(&scope);

    // Validate that all requested scopes are valid system scopes
    let valid_scopes = repo
        .validate_scopes(&requested_scopes)
        .await
        .map_err(|e| anyhow::anyhow!("Invalid scopes requested: {e}"))?;

    // Validate that all requested scopes are allowed for this client
    for requested_scope in &valid_scopes {
        if !client.scopes.contains(requested_scope) {
            return Err(anyhow::anyhow!(
                "Scope '{}' not allowed for client '{}'",
                requested_scope,
                params.client_id
            ));
        }
    }

    Ok(scope)
}

fn generate_webauthn_form(params: &AuthorizeQuery, resolved_scope: &str) -> String {
    let template = TemplateEngine::load_webauthn_oauth_template();
    let mut context = HashMap::new();

    let redirect_uri = params.redirect_uri.as_deref().unwrap_or_default();
    let state = params.state.as_deref().unwrap_or_default();
    let code_challenge = params.code_challenge.as_deref().unwrap_or_default();
    let code_challenge_method = params.code_challenge_method.as_deref().unwrap_or_default();

    context.insert("client_id", params.client_id.as_str());
    context.insert("scope", resolved_scope);
    context.insert("response_type", params.response_type.as_str());
    context.insert("redirect_uri", redirect_uri);
    context.insert("state", state);
    context.insert("code_challenge", code_challenge);
    context.insert("code_challenge_method", code_challenge_method);

    TemplateEngine::render(template, context)
}

/// Enhanced OAuth 2.0 parameter validation including PKCE and response modes
fn validate_oauth_parameters(params: &AuthorizeQuery) -> Result<(), String> {
    // Validate response_type
    if params.response_type != "code" {
        return Err(format!(
            "Unsupported response_type '{}'. Only 'code' is supported.",
            params.response_type
        ));
    }

    // Validate response_mode if provided (only query mode supported)
    if let Some(response_mode) = &params.response_mode {
        if response_mode != "query" {
            return Err(format!(
                "Unsupported response_mode '{response_mode}'. Only 'query' mode is supported."
            ));
        }
    }

    // Comprehensive PKCE validation (RFC 7636)
    if let Some(code_challenge) = &params.code_challenge {
        // PKCE code_challenge provided - validate format and security requirements
        if code_challenge.len() < 43 {
            return Err(
                "code_challenge too short. Must be at least 43 characters for security."
                    .to_string(),
            );
        }
        if code_challenge.len() > 128 {
            return Err("code_challenge too long. Must be at most 128 characters.".to_string());
        }

        // Validate code_challenge format (base64url-encoded)
        let is_valid_base64url = code_challenge
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

        if !is_valid_base64url {
            return Err(
                "code_challenge must be base64url encoded (A-Z, a-z, 0-9, -, _)".to_string(),
            );
        }

        // Check for sufficient entropy (no repeated patterns)
        if is_low_entropy_challenge(code_challenge) {
            return Err(
                "code_challenge appears to have insufficient entropy for security".to_string(),
            );
        }

        // Validate and enforce secure code_challenge_method - S256 only
        let method = params.code_challenge_method.as_deref().ok_or_else(|| {
            "code_challenge_method is required when code_challenge is provided".to_string()
        })?;

        match method {
            "S256" => {
                // Only allowed method - SHA256 hash provides security
            },
            "plain" => {
                return Err(
                    "PKCE method 'plain' is not allowed. Use 'S256' for security.".to_string(),
                );
            },
            _ => {
                return Err(format!(
                    "Unsupported code_challenge_method '{method}'. Only 'S256' is allowed."
                ));
            },
        }
    } else {
        // PKCE not provided - log for security monitoring
        eprintln!("ℹ️  Authorization request without PKCE from client {}. Consider requiring PKCE for enhanced security.",
            params.client_id);
    }

    // Validate display parameter if provided
    if let Some(display) = &params.display {
        match display.as_str() {
            "page" | "popup" | "touch" | "wap" => {
                // Valid display values
            },
            _ => {
                return Err(format!(
                    "Unsupported display value '{display}'. Supported values: page, popup, touch, wap."
                ));
            },
        }
    }

    // Validate prompt parameter if provided
    if let Some(prompt) = &params.prompt {
        for prompt_value in prompt.split_whitespace() {
            match prompt_value {
                "none" | "login" | "consent" | "select_account" => {
                    // Valid prompt values
                },
                _ => {
                    return Err(format!("Unsupported prompt value '{prompt_value}'. Supported values: none, login, consent, select_account."));
                },
            }
        }
    }

    // Validate max_age if provided
    if let Some(max_age) = params.max_age {
        if max_age < 0 {
            return Err("max_age must be a non-negative integer".to_string());
        }
    }

    Ok(())
}

/// Check for low entropy in PKCE code challenges to prevent weak values
fn is_low_entropy_challenge(challenge: &str) -> bool {
    // Check for obvious patterns that indicate low entropy

    // 1. All same character
    if challenge
        .chars()
        .all(|c| c == challenge.chars().next().unwrap())
    {
        return true;
    }

    // 2. Simple repeating patterns (like "abcabcabc...")
    if challenge.len() >= 6 {
        let pattern_length = 3;
        let pattern = &challenge[..pattern_length];
        let repetitions = challenge.len() / pattern_length;
        if repetitions > 2 && challenge.starts_with(&pattern.repeat(repetitions)) {
            return true;
        }
    }

    // 3. Sequential characters (like "abcdefgh..." or "12345678...")
    let chars: Vec<char> = challenge.chars().collect();
    if chars.len() >= 8 {
        let mut sequential_count = 1;
        for i in 1..chars.len() {
            if let (Some(prev), Some(curr)) = (chars[i - 1].to_digit(36), chars[i].to_digit(36)) {
                if curr == prev + 1 {
                    sequential_count += 1;
                    if sequential_count >= 8 {
                        return true; // Too many sequential characters
                    }
                } else {
                    sequential_count = 1;
                }
            }
        }
    }

    // 4. Check character distribution - ensure reasonable variety
    let unique_chars: std::collections::HashSet<char> = challenge.chars().collect();
    let entropy_ratio = unique_chars.len() as f64 / challenge.len() as f64;

    // If less than 30% unique characters, consider it low entropy
    if entropy_ratio < 0.3 {
        return true;
    }

    false
}
