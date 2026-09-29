//! Signed per-user identity for external MCP servers that trust this instance.
//!
//! An external server whose `external_auth.token_endpoint` names
//! [`accessor_path`] receives, on every proxied call, a short-lived RS256 JWT
//! signed with the instance authority key. It verifies the token against
//! `/.well-known/jwks.json` and reads who is calling; no provider credential
//! and no systemprompt session token ever leaves the gateway.

use crate::error::{AdminError, AdminResult};
use jsonwebtoken::{Algorithm, Header, encode};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;
use systemprompt::models::ServicesConfig;
use systemprompt::models::mcp::McpServerType;

pub const TOKEN_TTL_SECS: i64 = 300;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityClaims {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub name: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
}

#[must_use]
pub fn accessor_path(server: &str) -> String {
    format!("/api/public/identity/{server}/token")
}

// Why: `None` unless the server is enabled, external and names this accessor,
// so the accessor cannot sign for an id that never opted in or is switched off.
#[must_use]
pub fn audience_for(services: &ServicesConfig, server: &str) -> Option<String> {
    let deployment = services.mcp_servers.get(server)?;
    if !deployment.enabled || deployment.server_type != McpServerType::External {
        return None;
    }
    let auth = deployment.external_auth.as_ref()?;
    if auth.token_endpoint != accessor_path(server) {
        return None;
    }
    deployment.endpoint.clone()
}

#[must_use]
pub fn claims(
    issuer: &str,
    audience: &str,
    user: &UserId,
    email: &str,
    name: &str,
) -> IdentityClaims {
    let iat = chrono::Utc::now().timestamp();
    IdentityClaims {
        iss: issuer.to_owned(),
        aud: audience.to_owned(),
        sub: user.as_str().to_owned(),
        email: email.to_owned(),
        name: name.to_owned(),
        iat,
        exp: iat + TOKEN_TTL_SECS,
        jti: uuid::Uuid::new_v4().to_string(),
    }
}

pub fn sign(claims: &IdentityClaims) -> AdminResult<String> {
    use systemprompt_security::keys::authority;
    let mut header = Header::new(Algorithm::RS256);
    header.typ = Some("JWT".into());
    header.kid = Some(
        authority::active_kid()
            .map_err(AdminError::internal)?
            .to_owned(),
    );
    encode(
        &header,
        claims,
        authority::encoding_key().map_err(AdminError::internal)?,
    )
    .map_err(AdminError::internal)
}
