//! Per-user shareable manifest export.
//!
//! Issuance: admins POST `/admin/users/:id/share-token` to mint a signed token
//! that encodes the `user_id`, the current `users.share_token_version`, and an
//! expiry instant. Rotating the version revokes every previously-issued token;
//! the expiry bounds a token nobody remembered to revoke (REQ-025) — this is
//! the platform's most external-facing credential and it must not be the only
//! one without a clock.
//!
//! Verification: GET `/share/manifest/:token` is **public** (no auth
//! middleware). The token is validated, the version is rechecked against the
//! database, and the user's permissioned catalog is returned as JSON. The
//! filtering reuses `repositories::users::access_control::resolve_user_matrix`.

use crate::env::AdminEnv;

use std::sync::Arc;

use std::path::Path as FsPath;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::traits::ShareTokenSigner;

use crate::error::{AdminError, AdminResult};
use crate::handlers::shared;
use crate::repositories;
use crate::types::UserContext;

// Why: thirty days — long enough that a shared catalog link survives a
// procurement cycle, short enough that a forgotten one dies on its own.
const SHARE_TOKEN_TTL_DAYS: i64 = 30;

const SHARE_PURPOSE: &str = "share.manifest";

fn sign(signer: &dyn ShareTokenSigner, user_id: &UserId, version: i32) -> AdminResult<String> {
    let payload = format!("{user_id}:{version}");
    let ttl =
        std::time::Duration::from_secs(u64::try_from(SHARE_TOKEN_TTL_DAYS).unwrap_or(30) * 86_400);
    signer
        .sign(SHARE_PURPOSE, payload.as_bytes(), ttl)
        .map_err(AdminError::internal)
}

// Why: an expired, tampered or foreign token all verify as `None`, so the
// public endpoint gives an anonymous caller one answer, not three.
fn verify(signer: &dyn ShareTokenSigner, token: &str) -> Option<(UserId, i32)> {
    let payload = signer.verify(SHARE_PURPOSE, token).ok()?;
    let payload = String::from_utf8(payload).ok()?;
    let (user_id, version) = payload.rsplit_once(':')?;
    Some((UserId::new(user_id), version.parse().ok()?))
}

#[derive(Debug, Serialize)]
struct ShareTokenResponse {
    token: String,
    url: String,
}

pub(crate) async fn issue_share_token_handler(
    Extension(user_ctx): Extension<UserContext>,
    Extension(env): Extension<AdminEnv>,
    State(pool): State<Arc<PgPool>>,
    Path(target_user_id): Path<String>,
) -> AdminResult<Response> {
    // Why: `admin`, not `is_console`. A share token is a signed bearer for
    // someone else's manifest, so issuing one is credential minting.
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let target_user_id = UserId::new(target_user_id);
    let version = repositories::users::find_share_token_version(&pool, &target_user_id)
        .await?
        .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;
    let token = sign(env.share_tokens()?, &target_user_id, version)?;
    let url = format!("/share/manifest/{token}");
    Ok(Json(ShareTokenResponse { token, url }).into_response())
}

#[derive(Debug, Serialize)]
struct ManifestSection {
    entity_type: String,
    label: String,
    items: Vec<ManifestItem>,
}

#[derive(Debug, Serialize)]
struct ManifestItem {
    entity_id: String,
    entity_name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct ManifestResponse {
    user_id: UserId,
    sections: Vec<ManifestSection>,
}

pub(crate) async fn public_manifest_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(env): Extension<AdminEnv>,
    Path(token): Path<String>,
) -> AdminResult<Response> {
    let Some((user_id, version)) = verify(env.share_tokens()?, &token) else {
        return Err(AdminError::Unauthorized(
            "Invalid or revoked token".to_owned(),
        ));
    };

    // Why: this endpoint is unauthenticated, so a lookup failure of any kind —
    // missing row or database fault — answers the same way. Distinguishing them
    // would tell an anonymous caller which user ids exist.
    let current_version = match repositories::users::find_share_token_version(&pool, &user_id).await
    {
        Ok(Some(v)) => v,
        Ok(None) => {
            return Err(AdminError::Unauthorized(
                "Invalid or revoked token".to_owned(),
            ));
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load share_token_version for verification");
            return Err(AdminError::Unauthorized(
                "Invalid or revoked token".to_owned(),
            ));
        },
    };
    if current_version != version {
        return Err(AdminError::Unauthorized(
            "Token has been revoked".to_owned(),
        ));
    }

    Ok(Json(build_user_manifest(&env, &pool, &user_id).await?).into_response())
}

fn opt_desc(desc: String) -> Option<String> {
    if desc.is_empty() { None } else { Some(desc) }
}

fn collect_manifest_sections(
    env: &AdminEnv,
    services_path: &FsPath,
) -> Vec<repositories::users::access_control::SectionInput> {
    let mut sections_in: Vec<repositories::users::access_control::SectionInput> = Vec::new();

    if let Ok(servers) = repositories::mcp::mcp_servers::list_mcp_servers(services_path) {
        let rows = servers
            .into_iter()
            .map(|s| {
                let id = s.id.as_str().to_owned();
                (id.clone(), id, opt_desc(s.description))
            })
            .collect();
        sections_in.push(("mcp_server".into(), "MCP servers".into(), rows));
    }
    if let Ok(plugins) = repositories::marketplace::plugins::list_plugin_catalog(env, services_path)
    {
        let rows = plugins
            .into_iter()
            .map(|p| (p.id.as_str().to_owned(), p.name, opt_desc(p.description)))
            .collect();
        sections_in.push(("plugin".into(), "Plugins".into(), rows));
    }
    if let Ok(agents) = repositories::marketplace::plugins::list_agent_catalog(services_path) {
        let rows = agents
            .into_iter()
            .map(|a| (a.id.as_str().to_owned(), a.name, opt_desc(a.description)))
            .collect();
        sections_in.push(("agent".into(), "Agents".into(), rows));
    }
    if let Ok(skills) = repositories::marketplace::plugins::list_skill_catalog(services_path) {
        let rows = skills
            .into_iter()
            .map(|s| (s.id.as_str().to_owned(), s.name, opt_desc(s.description)))
            .collect();
        sections_in.push(("skill".into(), "Skills".into(), rows));
    }

    sections_in
}

async fn build_user_manifest(
    env: &AdminEnv,
    pool: &PgPool,
    user_id: &UserId,
) -> AdminResult<ManifestResponse> {
    let services_path = shared::get_services_path(env);
    let sections_in = collect_manifest_sections(env, &services_path);

    let matrix = repositories::users::access_control::filter_catalog_for_user(
        env,
        pool,
        user_id,
        sections_in,
    )
    .await?
    .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;

    let sections = matrix
        .sections
        .into_iter()
        .map(|sec| ManifestSection {
            entity_type: sec.entity_type,
            label: sec.label,
            items: sec
                .rows
                .into_iter()
                .filter(|r| r.effective == "allow")
                .map(|r| ManifestItem {
                    entity_id: r.entity_id,
                    entity_name: r.entity_name,
                    description: r.description,
                })
                .collect(),
        })
        .collect();

    Ok(ManifestResponse {
        user_id: UserId::new(matrix.user.id),
        sections,
    })
}
