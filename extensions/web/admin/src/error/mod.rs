//! The admin plane's HTTP error type.
//!
//! Domain errors convert in via `From`, and the variant alone decides the
//! status code, so handlers propagate with a bare `?` rather than mapping at
//! each call site. Logging happens once, in `into_response`.

mod html;
mod managed;
mod orchestration;

pub use html::{AdminHtmlError, AdminHtmlResult};

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::handlers::shared::ErrorBody;
use crate::repositories::bridge::BridgeRepoError;
use crate::repositories::secrets::secret_crypto::SecretCryptoError;
use crate::templates::AdminTemplateError;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unprocessable request: {0}")]
    Unprocessable(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    // Why: an uploaded archive the zip reader refuses is the client's to fix,
    // so the reader's own diagnosis is the 422 detail rather than a flattened
    // message that would hide which entry broke.
    #[error("Archive unreadable: {0}")]
    Archive(#[from] zip::result::ZipError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    // Why: Credentials were rejected. The cause is logged in full; the client is
    // told only that it failed, so token internals never reach the wire.
    #[error("Authentication failed: {0}")]
    Unauthenticated(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Too many requests: {0}")]
    RateLimited(String),

    // Why: A dependency this endpoint needs is not configured or not reachable.
    // Distinct from `Internal`: the request was well-formed and the server is
    // healthy, so a caller may sensibly retry or fall back.
    #[error("Unavailable: {0}")]
    Unavailable(String),

    // Why: An upstream this endpoint proxies to answered badly. Kept apart from
    // `Internal` so a caller can tell which side actually failed.
    #[error("Upstream error: {0}")]
    Upstream(String),

    // Why: A 400 whose cause is a typed validation error. The source's own
    // words are the client-facing detail, so the cause is kept rather than
    // flattened into the message.
    #[error("Invalid input: {context}: {source}")]
    Invalid {
        context: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Access token repository error: {0}")]
    BridgeRepo(BridgeRepoError),

    #[error("Marketplace error: {0}")]
    Marketplace(MarketplaceError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] SecretCryptoError),

    // Why: a route that exists in the router but whose handler has not
    // landed yet. Distinct from `NotFound`, which would tell a client the
    // endpoint is wrong when it is merely unfinished.
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl From<AdminTemplateError> for AdminError {
    fn from(value: AdminTemplateError) -> Self {
        Self::Internal(Box::new(value))
    }
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
    pub fn invalid<E>(context: &'static str, source: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Invalid {
            context,
            source: source.into(),
        }
    }

    #[must_use]
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) | Self::Marketplace(MarketplaceError::NotFound(_)) => {
                StatusCode::NOT_FOUND
            },
            Self::BadRequest(_)
            | Self::Invalid { .. }
            | Self::BridgeRepo(BridgeRepoError::Validation(_))
            | Self::Marketplace(MarketplaceError::BadRequest(_)) => StatusCode::BAD_REQUEST,
            Self::Unprocessable(_) | Self::Archive(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthorized(_) | Self::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) | Self::Marketplace(MarketplaceError::Conflict(_)) => {
                StatusCode::CONFLICT
            },
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            Self::Database(_)
            | Self::BridgeRepo(_)
            | Self::Marketplace(_)
            | Self::Crypto(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub(super) fn public_message(&self) -> String {
        match self {
            Self::NotFound(msg)
            | Self::Unprocessable(msg)
            | Self::BadRequest(msg)
            | Self::Unauthorized(msg)
            | Self::Forbidden(msg)
            | Self::Conflict(msg)
            | Self::RateLimited(msg)
            | Self::Unavailable(msg)
            | Self::NotImplemented(msg)
            | Self::BridgeRepo(BridgeRepoError::Validation(msg))
            | Self::Marketplace(
                MarketplaceError::BadRequest(msg)
                | MarketplaceError::NotFound(msg)
                | MarketplaceError::Conflict(msg),
            ) => msg.clone(),
            Self::Invalid { context, source } => format!("{context}: {source}"),
            Self::Archive(source) => format!("archive unreadable: {source}"),
            Self::Upstream(_) => "Upstream service error".to_owned(),
            Self::Unauthenticated(_) => "Unauthorized".to_owned(),
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

impl AdminError {
    #[must_use]
    pub fn unauthenticated<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Unauthenticated(err.into())
    }
}

impl From<systemprompt::oauth::OauthError> for AdminError {
    fn from(value: systemprompt::oauth::OauthError) -> Self {
        Self::Unauthenticated(Box::new(value))
    }
}

impl From<systemprompt::security::authz::AuthzError> for AdminError {
    fn from(value: systemprompt::security::authz::AuthzError) -> Self {
        match value {
            systemprompt::security::authz::AuthzError::Validation(msg) => Self::BadRequest(msg),
            other => Self::Internal(Box::new(other)),
        }
    }
}

impl From<systemprompt::models::errors::ConfigError> for AdminError {
    fn from(value: systemprompt::models::errors::ConfigError) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl From<systemprompt::config::ProfileBootstrapError> for AdminError {
    fn from(value: systemprompt::config::ProfileBootstrapError) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl From<systemprompt::loader::ConfigLoadError> for AdminError {
    fn from(value: systemprompt::loader::ConfigLoadError) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl AdminError {
    // Why: Record the failure once, at the boundary, at the severity its class
    // deserves. Both response faces call this, so a page failure and an API
    // failure leave the same trail.
    pub(super) fn log(&self, status: StatusCode) {
        if status.is_server_error() {
            tracing::error!(error = %self, "Admin handler returned server error");
        } else {
            tracing::warn!(error = %self, "Admin handler returned client error");
        }
    }
}

impl IntoResponse for AdminError {
    fn into_response(self) -> Response {
        let status = self.status();
        self.log(status);
        let body = Json(ErrorBody {
            error: self.public_message(),
        });
        (status, body).into_response()
    }
}

pub type AdminResult<T> = Result<T, AdminError>;
