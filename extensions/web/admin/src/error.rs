use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::repositories::bridge_grp::BridgeRepoError;
use crate::repositories::secret_crypto::SecretCryptoError;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Bridge repository error: {0}")]
    BridgeRepo(BridgeRepoError),

    #[error("Marketplace error: {0}")]
    Marketplace(MarketplaceError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] SecretCryptoError),

    #[error("Internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl AdminError {
    #[must_use]
    pub fn internal<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Internal(err.into())
    }

    #[must_use]
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) | Self::Marketplace(MarketplaceError::NotFound(_)) => {
                StatusCode::NOT_FOUND
            },
            Self::BadRequest(_)
            | Self::BridgeRepo(BridgeRepoError::Validation(_))
            | Self::Marketplace(MarketplaceError::BadRequest(_)) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_)
            | Self::BridgeRepo(_)
            | Self::Marketplace(_)
            | Self::Crypto(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn public_message(&self) -> String {
        match self {
            Self::NotFound(msg)
            | Self::BadRequest(msg)
            | Self::Unauthorized(msg)
            | Self::Forbidden(msg)
            | Self::Conflict(msg)
            | Self::BridgeRepo(BridgeRepoError::Validation(msg))
            | Self::Marketplace(
                MarketplaceError::BadRequest(msg) | MarketplaceError::NotFound(msg),
            ) => msg.clone(),
            Self::Crypto(_) => "Internal configuration error".to_owned(),
            Self::Database(_) | Self::BridgeRepo(_) | Self::Marketplace(_) | Self::Internal(_) => {
                "Internal server error".to_owned()
            },
        }
    }
}

impl From<BridgeRepoError> for AdminError {
    fn from(value: BridgeRepoError) -> Self {
        Self::BridgeRepo(value)
    }
}

impl From<MarketplaceError> for AdminError {
    fn from(value: MarketplaceError) -> Self {
        Self::Marketplace(value)
    }
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!(error = %self, "Admin handler returned server error");
        } else {
            tracing::warn!(error = %self, "Admin handler returned client error");
        }
        let body = Json(serde_json::json!({ "error": self.public_message() }));
        (status, body).into_response()
    }
}

pub type AdminResult<T> = Result<T, AdminError>;
