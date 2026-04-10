use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, thiserror::Error)]
pub enum GitSyncError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Git error: {0}")]
    Git(String),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Marketplace error: {0}")]
    Marketplace(#[from] MarketplaceError),
}
