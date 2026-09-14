//! Startup failures of the relay; every request-time failure is an HTTP
//! status, never a process exit.

#[derive(Debug, thiserror::Error)]
pub(crate) enum RelayError {
    #[error("SYSTEMPROMPT_RELAY_UPSTREAM is not set")]
    MissingUpstream { source: std::env::VarError },
    #[error("relay upstream must be a credential-free HTTP(S) origin")]
    InsecureUpstream,
    #[error("relay upstream is not a URL")]
    Url(#[from] url::ParseError),
    #[error("relay HTTP client could not be built")]
    Client(#[from] reqwest::Error),
    #[error("relay listener failed")]
    Io(#[from] std::io::Error),
}
