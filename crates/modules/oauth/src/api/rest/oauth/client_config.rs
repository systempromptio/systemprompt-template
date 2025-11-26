use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};

use crate::models::oauth::dynamic_registration::{
    DynamicRegistrationRequest, DynamicRegistrationResponse,
};
use crate::repository::OAuthRepository;

/// RFC 7591 - Client Configuration Read Endpoint
/// GET /`register/{client_id`}
pub async fn get_client_configuration(
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    // Validate registration access token from Authorization header
    let auth_header = match headers.get("authorization") {
        Some(header) => match header.to_str() {
            Ok(value) => value,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "invalid_token",
                        "error_description": "Invalid authorization header format"
                    })),
                )
                    .into_response();
            },
        },
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "invalid_token",
                    "error_description": "Missing authorization header"
                })),
            )
                .into_response();
        },
    };

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "invalid_token",
                "error_description": "Authorization header must use Bearer scheme"
            })),
        )
            .into_response();
    };

    // For now, accept any registration token starting with "reg_"
    // In production, you should validate against stored tokens
    if !token.starts_with("reg_") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "invalid_token",
                "error_description": "Invalid registration access token format"
            })),
        )
            .into_response();
    }

    // Fetch client configuration
    match repository.find_client(&client_id).await {
        Ok(Some(client)) => {
            let base_url = systemprompt_core_system::Config::global()
                .api_server_url
                .clone();

            let response = DynamicRegistrationResponse {
                client_id: client.client_id,
                client_secret: "***REDACTED***".to_string(), // Never return actual secret
                client_name: client.client_name,
                redirect_uris: client.redirect_uris,
                grant_types: client.grant_types,
                response_types: client.response_types,
                scope: client.scopes.join(" "),
                token_endpoint_auth_method: client.token_endpoint_auth_method,
                client_uri: client.client_uri,
                logo_uri: client.logo_uri,
                contacts: client.contacts,
                client_secret_expires_at: 0,
                client_id_issued_at: client.created_at,
                registration_access_token: token.to_string(),
                registration_client_uri: format!(
                    "{base_url}/api/v1/core/oauth/register/{client_id}"
                ),
            };

            (StatusCode::OK, Json(response)).into_response()
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "invalid_client_metadata",
                "error_description": "Client not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "server_error",
                "error_description": format!("Database error: {}", e)
            })),
        )
            .into_response(),
    }
}

/// RFC 7591 - Client Configuration Update Endpoint
/// PUT /`register/{client_id`}
pub async fn update_client_configuration(
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<DynamicRegistrationRequest>,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    // Same token validation as GET endpoint
    let registration_token = match validate_registration_token(&headers) {
        Ok(token) => token,
        Err(response) => return *response,
    };

    // Validate client exists
    let existing_client = match repository.find_client(&client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "invalid_client_metadata",
                    "error_description": "Client not found"
                })),
            )
                .into_response();
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "server_error",
                    "error_description": format!("Database error: {}", e)
                })),
            )
                .into_response();
        },
    };

    // Update client configuration
    let client_name = match request.get_client_name() {
        Ok(name) => name,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_client_metadata",
                    "error_description": e
                })),
            )
                .into_response();
        },
    };
    let redirect_uris = match request.get_redirect_uris() {
        Ok(uris) => {
            // Ensure both /oauth/callback variants are preserved
            let mut merged_uris = uris;
            let callback_base = "http://localhost:6274/oauth/callback";
            let callback_debug = "http://localhost:6274/oauth/callback/debug";

            // If we have one variant, ensure we have both
            if merged_uris
                .iter()
                .any(|uri| uri == callback_base || uri == callback_debug)
            {
                if !merged_uris.contains(&callback_base.to_string()) {
                    merged_uris.push(callback_base.to_string());
                }
                if !merged_uris.contains(&callback_debug.to_string()) {
                    merged_uris.push(callback_debug.to_string());
                }
            }

            // Sort and deduplicate
            merged_uris.sort();
            merged_uris.dedup();
            merged_uris
        },
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_client_metadata",
                    "error_description": e
                })),
            )
                .into_response();
        },
    };

    match repository
        .update_client(
            &client_id,
            Some(&client_name),
            Some(&redirect_uris),
            Some(&existing_client.scopes),
        )
        .await
    {
        Ok(_) => {
            let base_url = systemprompt_core_system::Config::global()
                .api_server_url
                .clone();

            let response = DynamicRegistrationResponse {
                client_id: client_id.clone(),
                client_secret: "***REDACTED***".to_string(),
                client_name,
                redirect_uris,
                grant_types: existing_client.grant_types,
                response_types: existing_client.response_types,
                scope: existing_client.scopes.join(" "),
                token_endpoint_auth_method: existing_client.token_endpoint_auth_method,
                client_uri: request.client_uri,
                logo_uri: request.logo_uri,
                contacts: request.contacts,
                client_secret_expires_at: 0,
                client_id_issued_at: existing_client.created_at,
                registration_access_token: registration_token.clone(),
                registration_client_uri: format!(
                    "{base_url}/api/v1/core/oauth/register/{client_id}"
                ),
            };

            (StatusCode::OK, Json(response)).into_response()
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_client_metadata",
                "error_description": format!("Failed to update client: {}", e)
            })),
        )
            .into_response(),
    }
}

/// RFC 7591 - Client Configuration Delete Endpoint
/// DELETE /`register/{client_id`}
pub async fn delete_client_configuration(
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    if let Err(response) = validate_registration_token(&headers) {
        return *response;
    }

    // Verify client exists before attempting deletion
    match repository.find_client(&client_id).await {
        Ok(Some(_)) => {
            // Client exists, proceed with deletion
            match repository.delete_client(&client_id).await {
                Ok(_) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "server_error",
                        "error_description": format!("Failed to delete client: {}", e)
                    })),
                )
                    .into_response(),
            }
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "invalid_client_metadata",
                "error_description": "Client not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "server_error",
                "error_description": format!("Database error: {}", e)
            })),
        )
            .into_response(),
    }
}

fn validate_registration_token(
    headers: &HeaderMap,
) -> Result<String, Box<axum::response::Response>> {
    let auth_header = match headers.get("authorization") {
        Some(header) => match header.to_str() {
            Ok(value) => value,
            Err(_) => {
                return Err(Box::new(
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "error": "invalid_token",
                            "error_description": "Invalid authorization header format"
                        })),
                    )
                        .into_response(),
                ));
            },
        },
        None => {
            return Err(Box::new(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "invalid_token",
                        "error_description": "Missing authorization header"
                    })),
                )
                    .into_response(),
            ));
        },
    };

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return Err(Box::new(
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "invalid_token",
                    "error_description": "Authorization header must use Bearer scheme"
                })),
            )
                .into_response(),
        ));
    };

    if !token.starts_with("reg_") {
        return Err(Box::new(
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "invalid_token",
                    "error_description": "Invalid registration access token format"
                })),
            )
                .into_response(),
        ));
    }

    Ok(token.to_string())
}
