use super::clients;
use crate::api::rest;
use axum::routing::{get, post};
use axum::Router;

pub fn router() -> Router<systemprompt_core_system::AppContext> {
    Router::new()
        .merge(public_router())
        .merge(authenticated_router())
}

pub fn public_router() -> Router<systemprompt_core_system::AppContext> {
    Router::new()
        .route("/health", get(rest::health::handle_health_api))
        .route(
            "/session",
            post(rest::oauth::anonymous::generate_anonymous_token),
        )
        .route(
            "/webauthn/complete",
            get(rest::oauth::webauthn_complete::handle_webauthn_complete),
        )
        .route("/token", post(rest::oauth::token::handle_token))
        .route(
            "/authorize",
            get(rest::oauth::authorize::handle_authorize_get),
        )
        .route(
            "/authorize",
            post(rest::oauth::authorize::handle_authorize_post),
        )
        .route("/callback", get(rest::oauth::callback::handle_callback))
        .route("/register", post(rest::oauth::register::register_client))
        .route(
            "/register/{client_id}",
            get(rest::oauth::client_config::get_client_configuration),
        )
        .route(
            "/register/{client_id}",
            axum::routing::put(rest::oauth::client_config::update_client_configuration),
        )
        .route(
            "/register/{client_id}",
            axum::routing::delete(rest::oauth::client_config::delete_client_configuration),
        )
        .route(
            "/webauthn/register/start",
            post(rest::webauthn::register::start_register),
        )
        .route(
            "/webauthn/register/finish",
            post(rest::webauthn::register::finish_register),
        )
        .route(
            "/webauthn/auth/start",
            post(rest::webauthn::authenticate::start_auth),
        )
        .route(
            "/webauthn/auth/finish",
            post(rest::webauthn::authenticate::finish_auth),
        )
        .route(
            "/webauthn/dev-auth",
            post(rest::webauthn::authenticate::dev_auth),
        )
}

pub fn authenticated_router() -> Router<systemprompt_core_system::AppContext> {
    Router::new()
        .nest("/clients", clients::router())
        .route(
            "/introspect",
            post(rest::oauth::introspect::handle_introspect),
        )
        .route("/revoke", post(rest::oauth::revoke::handle_revoke))
        .route("/userinfo", get(rest::oauth::userinfo::handle_userinfo))
        .route("/consent", get(rest::oauth::consent::handle_consent_get))
        .route("/consent", post(rest::oauth::consent::handle_consent_post))
}
