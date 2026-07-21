//! The admin plane's HTTP error type.
//!
//! Domain errors convert in via `From`, and the variant alone decides the
//! status code, so handlers propagate with a bare `?` rather than mapping at
//! each call site. Logging happens once, in `into_response`.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use systemprompt_web_shared::html_escape;
use thiserror::Error;

use crate::handlers::shared::ErrorBody;
use crate::repositories::bridge::BridgeRepoError;
use crate::repositories::secrets::secret_crypto::SecretCryptoError;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Credentials were rejected. The cause is logged in full; the client is
    /// told only that it failed, so token internals never reach the wire.
    #[error("Authentication failed: {0}")]
    Unauthenticated(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Too many requests: {0}")]
    RateLimited(String),

    /// A dependency this endpoint needs is not configured or not reachable.
    /// Distinct from `Internal`: the request was well-formed and the server is
    /// healthy, so a caller may sensibly retry or fall back.
    #[error("Unavailable: {0}")]
    Unavailable(String),

    /// An upstream this endpoint proxies to answered badly. Kept apart from
    /// `Internal` so a caller can tell which side actually failed.
    #[error("Upstream error: {0}")]
    Upstream(String),

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
            Self::Unauthorized(_) | Self::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
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
            | Self::RateLimited(msg)
            | Self::Unavailable(msg)
            | Self::BridgeRepo(BridgeRepoError::Validation(msg))
            | Self::Marketplace(
                MarketplaceError::BadRequest(msg) | MarketplaceError::NotFound(msg),
            ) => msg.clone(),
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
    /// Reject a request whose credentials did not check out.
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

impl AdminError {
    /// Record the failure once, at the boundary, at the severity its class
    /// deserves. Both response faces call this, so a page failure and an API
    /// failure leave the same trail.
    fn log(&self, status: StatusCode) {
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

/// The HTML face of [`AdminError`], for the server-rendered admin pages.
///
/// A browser navigating to a page needs a page, not a JSON body — but the
/// status and the client-visible text come from the same classification either
/// way, so an SSR handler cannot accidentally disagree with an API handler
/// about what a given failure means. Unlike the hand-rolled error pages this
/// replaces, it renders [`AdminError::public_message`], so an internal cause
/// is logged rather than interpolated into the page.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct AdminHtmlError(pub AdminError);

impl IntoResponse for AdminHtmlError {
    fn into_response(self) -> Response {
        let status = self.0.status();
        self.0.log(status);
        // The heading names the class of failure and the paragraph names this
        // instance of it, which is what the hand-rolled pages did one at a
        // time — "Access Denied" over "Admin access required", "Not Found"
        // over which thing was not found.
        let body = Html(format!(
            "<h1>{}</h1><p>{}</p>",
            status.canonical_reason().unwrap_or("Error"),
            html_escape(&self.0.public_message())
        ));
        (status, body).into_response()
    }
}

impl AdminHtmlError {
    /// Wrap an arbitrary failure with no better classification than 500.
    #[must_use]
    pub fn internal<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self(AdminError::Internal(err.into()))
    }
}

/// `?` in an SSR handler goes through whatever `AdminError` already knows how
/// to absorb, so the two faces stay in step by construction.
impl<E: Into<AdminError>> From<E> for AdminHtmlError {
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

pub type AdminResult<T> = Result<T, AdminError>;

/// The SSR counterpart to [`AdminResult`].
pub type AdminHtmlResult<T> = Result<T, AdminHtmlError>;
