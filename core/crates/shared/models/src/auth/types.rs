use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::enums::UserType;
use super::permission::Permission;

pub const BEARER_PREFIX: &str = "Bearer ";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub permissions: Vec<Permission>,
}

impl AuthenticatedUser {
    pub const fn new(
        id: Uuid,
        username: String,
        email: Option<String>,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            permissions,
        }
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
            || self.permissions.iter().any(|p| p.implies(&permission))
    }

    pub fn is_admin(&self) -> bool {
        self.has_permission(Permission::Admin)
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    pub fn user_type(&self) -> UserType {
        if self.has_permission(Permission::Admin) {
            UserType::Admin
        } else if self.has_permission(Permission::User) {
            UserType::Standard
        } else if self.has_permission(Permission::Service) {
            UserType::Service
        } else {
            UserType::Anon
        }
    }

    pub fn email_or_default(&self) -> String {
        self.email
            .clone()
            .unwrap_or_else(|| format!("{}@systemprompt.local", self.username))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token format")]
    InvalidTokenFormat,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token signature invalid")]
    InvalidSignature,

    #[error("User not found")]
    UserNotFound,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Invalid OAuth request: {reason}")]
    InvalidRequest { reason: String },

    #[error("CSRF token (state) is required")]
    MissingState,

    #[error("Redirect URI is required and must be registered")]
    InvalidRedirectUri,

    #[error("PKCE code_challenge is required")]
    MissingCodeChallenge,

    #[error("PKCE method '{method}' not allowed (must be S256)")]
    WeakPkceMethod { method: String },

    #[error("Client ID {client_id} not found")]
    ClientNotFound { client_id: String },

    #[error("Scope '{scope}' is invalid")]
    InvalidScope { scope: String },

    #[error("Token revocation requires authenticated user")]
    UnauthenticatedRevocation,

    #[error("WebAuthn RP ID could not be determined")]
    InvalidRpId,

    #[error("Client registration validation failed: {reason}")]
    RegistrationFailed { reason: String },

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkceMethod {
    S256,
}

impl std::str::FromStr for PkceMethod {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "S256" => Ok(Self::S256),
            "plain" => Err(AuthError::WeakPkceMethod {
                method: s.to_string(),
            }),
            _ => Err(AuthError::InvalidRequest {
                reason: format!("Unknown PKCE method: {s}"),
            }),
        }
    }
}

impl std::fmt::Display for PkceMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S256 => write!(f, "S256"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantType {
    AuthorizationCode,
    RefreshToken,
    ClientCredentials,
}

impl std::str::FromStr for GrantType {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "authorization_code" => Ok(Self::AuthorizationCode),
            "refresh_token" => Ok(Self::RefreshToken),
            "client_credentials" => Ok(Self::ClientCredentials),
            _ => Err(AuthError::InvalidRequest {
                reason: format!("Unknown grant type: {s}"),
            }),
        }
    }
}

impl std::fmt::Display for GrantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthorizationCode => write!(f, "authorization_code"),
            Self::RefreshToken => write!(f, "refresh_token"),
            Self::ClientCredentials => write!(f, "client_credentials"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    Code,
    Token,
}

impl std::str::FromStr for ResponseType {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(Self::Code),
            "token" => Ok(Self::Token),
            _ => Err(AuthError::InvalidRequest {
                reason: format!("Unknown response type: {s}"),
            }),
        }
    }
}

impl std::fmt::Display for ResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Code => write!(f, "code"),
            Self::Token => write!(f, "token"),
        }
    }
}
