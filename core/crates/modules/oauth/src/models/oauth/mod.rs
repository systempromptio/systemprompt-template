pub mod api;
pub mod dynamic_registration;

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use systemprompt_models::auth::{
    parse_permissions, permissions_to_string, JwtAudience, Permission, RateLimitTier, TokenType,
    UserType,
};

pub use systemprompt_models::oauth::OAuthServerConfig as OAuthConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    // Standard JWT Claims (RFC 7519)
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    #[serde(
        serialize_with = "serialize_audiences",
        deserialize_with = "deserialize_audiences"
    )]
    pub aud: Vec<JwtAudience>,
    pub jti: String,

    // OAuth 2.0 Claims
    #[serde(
        serialize_with = "serialize_scope",
        deserialize_with = "deserialize_scope"
    )]
    pub scope: Vec<Permission>,

    // SystemPrompt User Claims
    pub username: String,
    pub email: String,
    pub user_type: UserType,

    // Enhanced Security Claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    pub token_type: TokenType,
    pub auth_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    // Rate Limiting & Security Metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_tier: Option<RateLimitTier>,
}

fn serialize_audiences<S>(auds: &[JwtAudience], s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = s.serialize_seq(Some(auds.len()))?;
    for aud in auds {
        seq.serialize_element(aud.as_str())?;
    }
    seq.end()
}

fn deserialize_audiences<'de, D>(d: D) -> Result<Vec<JwtAudience>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(d)?;
    strings
        .iter()
        .map(|s| JwtAudience::from_str(s).map_err(serde::de::Error::custom))
        .collect()
}

fn serialize_scope<S>(permissions: &[Permission], s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&permissions_to_string(permissions))
}

fn deserialize_scope<'de, D>(d: D) -> Result<Vec<Permission>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let scope_string: String = String::deserialize(d)?;
    parse_permissions(&scope_string).map_err(serde::de::Error::custom)
}

impl JwtClaims {
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.scope.contains(&permission)
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.scope
    }

    pub fn get_permissions(&self) -> Vec<Permission> {
        self.scope.clone()
    }

    pub fn get_scopes(&self) -> Vec<String> {
        self.scope.iter().map(ToString::to_string).collect()
    }

    pub fn is_admin(&self) -> bool {
        self.has_permission(Permission::Admin)
    }

    pub fn is_registered_user(&self) -> bool {
        self.has_permission(Permission::User)
    }

    pub fn is_anonymous(&self) -> bool {
        self.has_permission(Permission::Anonymous)
    }

    pub fn has_audience(&self, aud: JwtAudience) -> bool {
        self.aud.contains(&aud)
    }
}
