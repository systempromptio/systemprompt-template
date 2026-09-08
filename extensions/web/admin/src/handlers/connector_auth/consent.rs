//! Browser consent bound to the initiating login and connection generation.

use super::live_user;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use crate::services::connector_accounts as service;
use crate::services::connector_oauth::{self as oauth, Provider};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Deserialize)]
pub(super) struct Start {
    resource_id: Option<String>,
    expected_user: Option<String>,
}

pub(super) async fn start(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Query(params): Query<Start>,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, false).await?;
    if params
        .expected_user
        .is_some_and(|id| id != user.user_id.as_str())
    {
        return Err(AdminError::Forbidden(
            "Sign in to the same Systemprompt account as your bridge".into(),
        ));
    }
    service::require_entitlement(&pool, &user.user_id, provider).await?;
    let mut tx = pool.begin().await?;
    let account = accounts::get_locked_account(&mut tx, &user.user_id, provider.slug()).await?;
    tx.commit().await?;
    let state = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let verifier = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let (url, mut grant) =
        oauth::authorize(user.user_id.as_str(), provider, &state, verifier).await?;
    grant.session = user.session_id.map(|s| s.to_string());
    grant.generation = account.generation;
    grant.resource_id = params.resource_id.unwrap_or_default();
    credentials::save_state(
        &pool,
        &user.user_id,
        provider.slug(),
        &state,
        &oauth::seal(&grant)?,
    )
    .await?;
    Ok(([(CACHE_CONTROL, "no-store")], Redirect::to(&url)).into_response())
}

#[derive(Deserialize)]
pub(super) struct Callback {
    state: String,
    code: Option<String>,
    error: Option<String>,
}

pub(super) async fn callback(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Query(params): Query<Callback>,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, false).await?;
    service::require_entitlement(&pool, &user.user_id, provider).await?;
    let row = credentials::consume_state(&pool, &user.user_id, provider.slug(), &params.state)
        .await?
        .ok_or_else(|| {
            AdminError::Unauthorized("Connector consent expired or already consumed".into())
        })?;
    let mut grant = oauth::open(&row, &user.user_id, provider)?;
    if grant.session != user.session_id.as_ref().map(ToString::to_string) {
        return Err(AdminError::Unauthorized(
            "Login changed during connector consent".into(),
        ));
    }
    if params.error.is_some() {
        return Err(AdminError::Unauthorized("Connector consent denied".into()));
    }
    let code = params
        .code
        .ok_or_else(|| AdminError::BadRequest("Consent code missing".into()))?;
    oauth::exchange(&mut grant, &code).await?;
    if let Err(error) = oauth::verify::verify(&mut grant).await {
        if !matches!(error, AdminError::Unauthorized(_)) {
            live_user(&pool, &headers, false).await?;
            super::store_pending_verification(&pool, &user.user_id, &grant).await?;
        }
        return Err(error);
    }
    // Why: Recheck revocation after the network exchange, before persisting a
    // grant.
    live_user(&pool, &headers, false).await?;
    super::store_verified(&pool, &user.user_id, &grant).await?;
    tracing::info!(user_id = %user.user_id, provider = provider.slug(), "connector_connected");
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Redirect::to("/admin/profile#connected-accounts"),
    )
        .into_response())
}

pub(super) async fn token(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    use sha2::{Digest, Sha256};
    let expected = oauth::config::secret("mcp_credential_broker_secret")?;
    let presented = headers
        .get("x-systemprompt-credential-broker")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AdminError::Forbidden("Backend credential access required".into()))?;
    if Sha256::digest(expected.as_bytes()) != Sha256::digest(presented.as_bytes()) {
        return Err(AdminError::Forbidden(
            "Backend credential access rejected".into(),
        ));
    }
    let user = live_user(&pool, &headers, false).await?;
    service::require_entitlement(&pool, &user.user_id, provider).await?;
    let access_token = oauth::verified_token(&pool, &user.user_id, provider, false).await?;
    // Why: The descriptor uses an empty scheme: the trusted adapter supplies the
    // complete header so personal Atlassian tokens can use Basic authentication.
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({"access_token": access_token})),
    )
        .into_response())
}
