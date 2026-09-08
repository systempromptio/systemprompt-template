//! The typed accessor that mints a per-user Salesforce bearer for the
//! Salesforce MCP server.
//!
//! Not a login: ADFS is the login. A user's Salesforce Username is recorded
//! administratively (see [`crate::repositories::users::salesforce_identity`]),
//! and [`salesforce_token_handler`] mints a fresh bearer as that user on every
//! call via the RFC 7523 JWT-bearer grant. Nothing is banked.

mod config;
mod tokens;
mod unlink;

use std::sync::Arc;

use sqlx::PgPool;

pub use config::SalesforceConfig;
pub(crate) use config::salesforce_private_key;
pub(crate) use tokens::{post_token_request, salesforce_token_handler};
pub(crate) use unlink::salesforce_unlink;

/// Errors from the Salesforce token plumbing.
#[derive(Debug, thiserror::Error)]
pub enum SalesforceError {
    #[error("Salesforce HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Salesforce token endpoint returned {status}: {body}")]
    TokenEndpoint {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("SALESFORCE_PRIVATE_KEY is not set")]
    MissingPrivateKey,
    #[error("system clock before epoch: {0}")]
    Clock(#[from] std::time::SystemTimeError),
    #[error("SALESFORCE_PRIVATE_KEY is not a valid RSA private key: {0}")]
    PrivateKey(#[source] jsonwebtoken::errors::Error),
    #[error("assertion signing failed: {0}")]
    Signing(#[source] jsonwebtoken::errors::Error),
}

/// Per-request dependencies for the Salesforce accessor, shared via an axum
/// `Extension`.
#[derive(Clone)]
pub struct SalesforceDeps {
    pub config: Arc<SalesforceConfig>,
    pub write_pool: Arc<PgPool>,
}

impl std::fmt::Debug for SalesforceDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SalesforceDeps")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
