//! Per-user shareable manifest export.
//!
//! Issuance: admins POST `/admin/users/:id/share-token` to mint a signed token
//! that encodes the `user_id` and the current `users.share_token_version`.
//! Rotating the version revokes every previously-issued token.
//!
//! Verification: GET `/share/manifest/:token` is **public** (no auth
//! middleware). The token is validated, the version is rechecked against the
//! database, and the user's permissioned catalog is returned as JSON. The
//! filtering reuses `repositories::access_control::resolve_user_matrix`.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use base64::Engine;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::config::SecretsBootstrap;

use crate::handlers::shared;
use crate::repositories;
use crate::types::UserContext;

/// Build a signed token of the form `b64(user_id):b64(version):hex(hmac)`.
/// The HMAC follows RFC 2104 over the concatenation `user_id:version`, keyed
/// off the existing JWT signing secret. Reusing the JWT secret avoids
/// introducing a second piece of bootstrap config.
fn sign(secret: &[u8], user_id: &str, version: i32) -> String {
    let payload = format!("{user_id}:{version}");
    let mut padded = [0u8; 64];
    if secret.len() > 64 {
        let mut h = Sha256::new();
        h.update(secret);
        let digest = h.finalize();
        padded[..32].copy_from_slice(&digest);
    } else {
        padded[..secret.len()].copy_from_slice(secret);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= padded[i];
        opad[i] ^= padded[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(payload.as_bytes());
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    let mac = outer.finalize();

    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let uid_b64 = b64.encode(user_id.as_bytes());
    let ver_b64 = b64.encode(version.to_string().as_bytes());
    let mut mac_hex = String::with_capacity(mac.len() * 2);
    for b in mac {
        use std::fmt::Write;
        let _ = write!(mac_hex, "{b:02x}");
    }
    format!("{uid_b64}:{ver_b64}:{mac_hex}")
}

fn verify(secret: &[u8], token: &str) -> Option<(String, i32)> {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let user_id = String::from_utf8(b64.decode(parts[0]).ok()?).ok()?;
    let ver_s = String::from_utf8(b64.decode(parts[1]).ok()?).ok()?;
    let version: i32 = ver_s.parse().ok()?;
    let expected = sign(secret, &user_id, version);
    // Constant-time compare on the produced strings.
    if expected.len() != token.len() {
        return None;
    }
    let mut diff = 0u8;
    for (a, b) in expected.bytes().zip(token.bytes()) {
        diff |= a ^ b;
    }
    if diff == 0 {
        Some((user_id, version))
    } else {
        None
    }
}

#[derive(Debug, Serialize)]
struct ShareTokenResponse {
    token: String,
    url: String,
}

pub async fn issue_share_token_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(target_user_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin access required");
    }
    let secret = match SecretsBootstrap::jwt_secret() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT secret for share token");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load secret",
            );
        }
    };
    let row = sqlx::query_as::<_, (i32,)>("SELECT share_token_version FROM users WHERE id = $1")
        .bind(&target_user_id)
        .fetch_optional(&*pool)
        .await;
    let version = match row {
        Ok(Some((v,))) => v,
        Ok(None) => return shared::error_response(StatusCode::NOT_FOUND, "User not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load share_token_version");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        }
    };
    let token = sign(secret.as_bytes(), &target_user_id, version);
    let url = format!("/share/manifest/{token}");
    Json(ShareTokenResponse { token, url }).into_response()
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
    user_id: String,
    sections: Vec<ManifestSection>,
}

pub async fn public_manifest_handler(
    State(pool): State<Arc<PgPool>>,
    Path(token): Path<String>,
) -> Response {
    let secret = match SecretsBootstrap::jwt_secret() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT secret for manifest");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load secret",
            );
        }
    };
    let Some((user_id, version)) = verify(secret.as_bytes(), &token) else {
        return shared::error_response(StatusCode::UNAUTHORIZED, "Invalid or revoked token");
    };

    // Re-check version against DB to honour revocation via bumping the column.
    let current: Option<(i32,)> =
        sqlx::query_as::<_, (i32,)>("SELECT share_token_version FROM users WHERE id = $1")
            .bind(&user_id)
            .fetch_optional(&*pool)
            .await
            .unwrap_or(None);
    let Some((current_version,)) = current else {
        return shared::error_response(StatusCode::UNAUTHORIZED, "Invalid or revoked token");
    };
    if current_version != version {
        return shared::error_response(StatusCode::UNAUTHORIZED, "Token has been revoked");
    }

    match build_user_manifest(&pool, &user_id).await {
        Ok(m) => Json(m).into_response(),
        Err(r) => r,
    }
}

async fn build_user_manifest(pool: &PgPool, user_id: &str) -> Result<ManifestResponse, Response> {
    let services_path = shared::get_services_path().map_err(|r| *r)?;
    let mut sections_in: Vec<repositories::access_control::SectionInput> = Vec::new();

    if let Ok(servers) = repositories::mcp_servers::list_mcp_servers(&services_path) {
        let rows = servers
            .into_iter()
            .map(|s| {
                let id = s.id.as_str().to_string();
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (id.clone(), id, desc)
            })
            .collect();
        sections_in.push(("mcp_server".into(), "MCP servers".into(), rows));
    }
    if let Ok(plugins) = repositories::list_plugin_catalog(&services_path) {
        let rows = plugins
            .into_iter()
            .map(|p| {
                let desc = if p.description.is_empty() {
                    None
                } else {
                    Some(p.description)
                };
                (p.id, p.name, desc)
            })
            .collect();
        sections_in.push(("plugin".into(), "Plugins".into(), rows));
    }
    if let Ok(agents) = repositories::list_agent_catalog(&services_path) {
        let rows = agents
            .into_iter()
            .map(|a| {
                let desc = if a.description.is_empty() {
                    None
                } else {
                    Some(a.description)
                };
                (a.id.as_str().to_string(), a.name, desc)
            })
            .collect();
        sections_in.push(("agent".into(), "Agents".into(), rows));
    }
    if let Ok(skills) = repositories::list_skill_catalog(&services_path) {
        let rows = skills
            .into_iter()
            .map(|s| {
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (s.id.as_str().to_string(), s.name, desc)
            })
            .collect();
        sections_in.push(("skill".into(), "Skills".into(), rows));
    }

    let matrix = repositories::access_control::filter_catalog_for_user(pool, user_id, sections_in)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id, "Failed to resolve manifest matrix");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        })?;

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
        user_id: matrix.user.id,
        sections,
    })
}
