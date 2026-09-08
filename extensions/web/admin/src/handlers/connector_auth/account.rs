//! Account state and explicit mutations shared by the browser and bridge.

use super::live_user;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use crate::services::connector_accounts as service;
use crate::services::connector_oauth::{self as oauth, Provider};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

pub(super) async fn list(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, false).await?;
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(service::get_connections(&pool, &user.user_id).await?),
    )
        .into_response())
}

pub(super) async fn disconnect(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, true).await?;
    let mut tx = pool.begin().await?;
    let mut row = accounts::get_locked_account(&mut tx, &user.user_id, provider.slug()).await?;
    credentials::delete(&mut tx, &user.user_id, provider.slug()).await?;
    accounts::delete_pending(&mut tx, &user.user_id, provider.slug()).await?;
    row.status = "not_connected".into();
    row.auth_method = None;
    row.account_id = None;
    row.account_name = None;
    row.resource_id = None;
    row.resource_name = None;
    row.error_code = None;
    row.verified_at = None;
    row.generation += 1;
    accounts::update_account(&mut tx, &user.user_id, &row).await?;
    tx.commit().await?;
    tracing::info!(user_id = %user.user_id, provider = provider.slug(), "connector_disconnected");
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(service::get_connections(&pool, &user.user_id).await?),
    )
        .into_response())
}

pub(super) async fn test(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, true).await?;
    service::require_entitlement(&pool, &user.user_id, provider).await?;
    oauth::verified_token(&pool, &user.user_id, provider, true).await?;
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(service::get_connections(&pool, &user.user_id).await?),
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManualToken {
    token: String,
    email: Option<String>,
    resource_id: Option<String>,
}

pub(super) async fn manual(
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<Provider>,
    headers: HeaderMap,
    Json(input): Json<ManualToken>,
) -> AdminResult<Response> {
    let user = live_user(&pool, &headers, true).await?;
    service::require_entitlement(&pool, &user.user_id, provider).await?;
    if !provider.configured() || provider == Provider::Salesforce {
        return Err(AdminError::BadRequest(
            "Use browser authorization for this connector".into(),
        ));
    }
    if input.token.trim().is_empty()
        || input.token.len() > 16384
        || input.token.chars().any(char::is_control)
    {
        return Err(AdminError::BadRequest("Invalid personal token".into()));
    }
    let (authorization_scheme, access_token) = if provider == Provider::Atlassian {
        let email = input
            .email
            .filter(|s| s.contains('@') && !s.contains(':') && !s.chars().any(char::is_control))
            .ok_or_else(|| {
                AdminError::BadRequest("Atlassian personal token requires its owner's email".into())
            })?;
        ("Basic", STANDARD.encode(format!("{email}:{}", input.token)))
    } else {
        ("Bearer", input.token)
    };
    let mut tx = pool.begin().await?;
    let row = accounts::get_locked_account(&mut tx, &user.user_id, provider.slug()).await?;
    let generation = row.generation;
    tx.commit().await?;
    let mut grant = oauth::Grant {
        user: user.user_id.to_string(),
        provider,
        client: String::new(),
        client_secret: String::new(),
        verifier: String::new(),
        access_token,
        refresh_token: None,
        expires_at: i64::MAX,
        token_endpoint: String::new(),
        generation,
        session: None,
        auth_method: "personal_token".into(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: input.resource_id.unwrap_or_default(),
        resource_name: String::new(),
        authorization_scheme: authorization_scheme.into(),
    };
    oauth::verify::verify(&mut grant).await?;
    super::store_verified(&pool, &user.user_id, &grant).await?;
    tracing::info!(user_id = %user.user_id, provider = provider.slug(), "connector_personal_token_verified");
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(service::get_connections(&pool, &user.user_id).await?),
    )
        .into_response())
}

pub(super) fn apply_verified(row: &mut accounts::ProviderConnection, grant: &oauth::Grant) {
    row.status = "connected".into();
    row.auth_method = Some(grant.auth_method.clone());
    row.account_id = Some(grant.account_id.clone());
    row.account_name = Some(grant.account_name.clone());
    row.resource_id = Some(grant.resource_id.clone());
    row.resource_name = Some(grant.resource_name.clone());
    row.error_code = None;
    row.verified_at = Some(Utc::now());
}
