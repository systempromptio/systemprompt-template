//! Hosted MCP consent and account actions; credentials stay in the backend.

mod account;
mod consent;

use crate::error::{AdminError, AdminResult};
use crate::handlers::users::extract_mcp_accessor_user;
use crate::repositories::users::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use crate::services::connector_oauth as oauth;
use axum::Router;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;

pub fn router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/account/connections", get(account::list))
        .route(
            "/account/connections/{provider}/disconnect",
            post(account::disconnect),
        )
        .route("/account/connections/{provider}/test", post(account::test))
        .route(
            "/account/connections/{provider}/manual-token",
            post(account::manual),
        )
        .route("/connectors/{provider}/start", get(consent::start))
        .route("/connectors/{provider}/callback", get(consent::callback))
        .route("/connectors/{provider}/token", get(consent::token))
        .layer(axum::extract::DefaultBodyLimit::max(32768))
        .with_state(pool)
}

async fn live_user(
    pool: &PgPool,
    headers: &HeaderMap,
    mutation: bool,
) -> AdminResult<crate::types::CookieSession> {
    // Why: Cookie mutations require an exact same-origin browser request. Bridge
    // bearer requests do not use ambient cookies and are not subject to CSRF.
    if mutation && !headers.contains_key("authorization") {
        let origin = headers
            .get("origin")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AdminError::Forbidden("Same-origin request required".into()))?;
        let base = systemprompt::models::Config::get()?
            .api_external_url
            .clone();
        let expected = url::Url::parse(&base)
            .map_err(AdminError::internal)?
            .origin()
            .ascii_serialization();
        if origin != expected {
            return Err(AdminError::Forbidden(
                "Cross-origin account mutation rejected".into(),
            ));
        }
    }
    let user = extract_mcp_accessor_user(headers)?;
    let session = user
        .session_id
        .as_ref()
        .ok_or_else(|| AdminError::Unauthorized("A live login session is required".into()))?;
    if !accounts::is_live_session(pool, &user.user_id, session.as_str()).await? {
        return Err(AdminError::Unauthorized(
            "Login session is no longer active".into(),
        ));
    }
    Ok(user)
}

async fn store_verified(pool: &PgPool, user: &UserId, grant: &oauth::Grant) -> AdminResult<()> {
    let mut tx = pool.begin().await?;
    let mut row = accounts::get_locked_account(&mut tx, user, grant.provider.slug()).await?;
    if row.generation != grant.generation {
        return Err(AdminError::Conflict(
            "Connection changed during authorization; start again".into(),
        ));
    }
    credentials::store(&mut tx, user, grant.provider.slug(), &oauth::seal(grant)?).await?;
    account::apply_verified(&mut row, grant);
    row.generation += 1;
    accounts::update_account(&mut tx, user, &row).await?;
    accounts::delete_pending(&mut tx, user, grant.provider.slug()).await?;
    tx.commit().await?;
    Ok(())
}

async fn store_pending_verification(
    pool: &PgPool,
    user: &UserId,
    grant: &oauth::Grant,
) -> AdminResult<()> {
    let mut tx = pool.begin().await?;
    let mut row = accounts::get_locked_account(&mut tx, user, grant.provider.slug()).await?;
    if row.generation != grant.generation {
        return Err(AdminError::Conflict(
            "Connection changed during authorization; start again".into(),
        ));
    }
    credentials::store(&mut tx, user, grant.provider.slug(), &oauth::seal(grant)?).await?;
    row.status = "verification_required".into();
    row.auth_method = Some(grant.auth_method.clone());
    row.account_id = None;
    row.account_name = None;
    row.resource_id = None;
    row.resource_name = None;
    row.verified_at = None;
    row.error_code = Some("verification_failed".into());
    row.generation += 1;
    accounts::update_account(&mut tx, user, &row).await?;
    accounts::delete_pending(&mut tx, user, grant.provider.slug()).await?;
    tx.commit().await?;
    Ok(())
}
