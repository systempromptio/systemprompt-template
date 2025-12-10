use super::{AuthorizeQuery, AuthorizeRequest, AuthorizeResponse};
use crate::templates::TemplateEngine;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use std::collections::HashMap;

pub fn convert_form_to_query(form: &AuthorizeRequest) -> AuthorizeQuery {
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

pub fn is_user_consent_granted(form: &AuthorizeRequest) -> bool {
    form.user_consent.as_deref() == Some("allow")
}

pub fn create_consent_denied_response(query: &AuthorizeQuery) -> axum::response::Response {
    create_error_response(
        query,
        "access_denied",
        "User denied the request",
        StatusCode::UNAUTHORIZED,
    )
}

pub fn create_error_response(
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

pub fn generate_webauthn_form(params: &AuthorizeQuery, resolved_scope: &str) -> String {
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
