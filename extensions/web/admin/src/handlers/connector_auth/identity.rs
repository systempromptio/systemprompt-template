//! Broker-only accessor that signs the caller's identity for one external
//! server.

use super::{live_user, require_broker};
use crate::authz::catalog::CatalogAccess;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::queries::find_identity_envelope;
use crate::services::identity_token;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_security::authz::EntityKind;

pub(super) async fn token(
    State(pool): State<Arc<PgPool>>,
    Path(server): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    require_broker(&headers)?;
    let user = live_user(&pool, &headers, false).await?;
    let services = systemprompt::loader::ServicesBootstrap::get().map_err(AdminError::internal)?;
    let audience = identity_token::audience_for(services, &server)
        .ok_or_else(|| AdminError::NotFound("No identity-backed MCP server by that id".into()))?;
    let permitted = CatalogAccess::load(&pool, &user.user_id)
        .await?
        .allowed(EntityKind::McpServer, std::slice::from_ref(&server))
        .await?;
    if !permitted.contains(&server) {
        return Err(AdminError::Forbidden(
            "This account is not entitled to the MCP server".into(),
        ));
    }
    let identity = find_identity_envelope(&pool, &user.user_id)
        .await?
        .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
    let name = identity
        .display_name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(identity.username);
    let issuer = systemprompt::manifest::Config::get()?.jwt_issuer.clone();
    let claims = identity_token::claims(&issuer, &audience, &user.user_id, &identity.email, &name);
    let access_token = identity_token::sign(&claims)?;
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({"access_token": access_token})),
    )
        .into_response())
}
