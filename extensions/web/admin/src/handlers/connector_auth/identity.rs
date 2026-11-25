//! Broker-only accessor that signs the caller's identity for one external
//! server.

use crate::env::AdminEnv;

use super::{live_user, require_broker};
use crate::authz::catalog::CatalogAccess;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::queries::find_identity_envelope;
use crate::services::identity_token;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::governance::authz::EntityKind;

pub(super) async fn token(
    State(pool): State<Arc<PgPool>>,
    Extension(env): Extension<AdminEnv>,
    Path(server): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    require_broker(&env, &headers)?;
    let user = live_user(&env, &pool, &headers, false).await?;
    let audience = identity_token::audience_for(&env.services(), &server)
        .ok_or_else(|| AdminError::NotFound("No identity-backed MCP server by that id".into()))?;
    let permitted = CatalogAccess::load(&env, &pool, &user.user_id)
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
    let claims = identity_token::claims(
        &env.config().jwt_issuer,
        &audience,
        &user.user_id,
        &identity.email,
        &name,
    );
    let access_token = identity_token::sign(&claims)?;
    Ok((
        [(CACHE_CONTROL, "no-store")],
        Json(serde_json::json!({"access_token": access_token})),
    )
        .into_response())
}
