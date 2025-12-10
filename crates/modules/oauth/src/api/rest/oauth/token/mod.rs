mod generation;
mod handler;
mod validation;

pub use handler::handle_token;

use serde::{Deserialize, Serialize};

pub type TokenResult<T> = Result<T, TokenError>;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub code_verifier: Option<String>,
}

#[derive(Debug, Serialize)]

pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Invalid request: {field} {message}")]
    InvalidRequest { field: String, message: String },

    #[error("Unsupported grant type: {grant_type}")]
    UnsupportedGrantType { grant_type: String },

    #[error("Invalid client credentials")]
    InvalidClient,

    #[error("Invalid authorization code: {reason}")]
    InvalidGrant { reason: String },

    #[error("Invalid refresh token: {reason}")]
    InvalidRefreshToken { reason: String },

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid client secret")]
    InvalidClientSecret,

    #[error("Authorization code expired")]
    ExpiredCode,

    #[error("Server error: {message}")]
    ServerError { message: String },
}

#[derive(Debug, Serialize)]

pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

impl From<TokenError> for TokenErrorResponse {
    fn from(error: TokenError) -> Self {
        let (error_type, description) = match &error {
            TokenError::InvalidRequest { field, message } => {
                ("invalid_request", Some(format!("{field}: {message}")))
            },
            TokenError::UnsupportedGrantType { grant_type } => (
                "unsupported_grant_type",
                Some(format!("Grant type '{grant_type}' is not supported")),
            ),
            TokenError::InvalidClient => (
                "invalid_client",
                Some("Client authentication failed".to_string()),
            ),
            TokenError::InvalidGrant { reason } => ("invalid_grant", Some(reason.clone())),
            TokenError::InvalidRefreshToken { reason } => (
                "invalid_grant",
                Some(format!("Refresh token invalid: {reason}")),
            ),
            TokenError::InvalidCredentials => {
                ("invalid_grant", Some("Invalid credentials".to_string()))
            },
            TokenError::InvalidClientSecret => {
                ("invalid_client", Some("Invalid client secret".to_string()))
            },
            TokenError::ExpiredCode => (
                "invalid_grant",
                Some("Authorization code expired".to_string()),
            ),
            TokenError::ServerError { message } => ("server_error", Some(message.clone())),
        };

        Self {
            error: error_type.to_string(),
            error_description: description,
        }
    }
}
