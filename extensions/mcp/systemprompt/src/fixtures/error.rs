//! What a fixture read or write can fail with, and how each reason reaches
//! the MCP client. Database and canonicalisation failures stay typed here so
//! the log carries the cause while the client sees only that the server
//! failed; a rejected request names the rule it broke.

use rmcp::ErrorData as McpError;
use systemprompt::identifiers::EvalApprovalId;

#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    #[error("{0}")]
    Rejected(&'static str),
    #[error("{0}")]
    Unavailable(&'static str),
    #[error("Approval required: {0}")]
    ApprovalRequired(EvalApprovalId),
    #[error("evaluation database failed")]
    Database(#[from] sqlx::Error),
    #[error("evaluation database pool failed")]
    Pool(#[from] systemprompt::database::RepositoryError),
    #[error("evaluation lifecycle failed")]
    Lifecycle(#[from] systemprompt::evaluation::EvaluationError),
    #[error("fixture payload cannot be canonicalised")]
    Canonicalize(#[from] serde_json::Error),
}

impl From<FixtureError> for McpError {
    fn from(error: FixtureError) -> Self {
        match error {
            FixtureError::Rejected(reason) => Self::invalid_params(reason, None),
            FixtureError::ApprovalRequired(_) => Self::invalid_params(error.to_string(), None),
            FixtureError::Unavailable(reason) => Self::internal_error(reason, None),
            FixtureError::Database(_)
            | FixtureError::Pool(_)
            | FixtureError::Lifecycle(_)
            | FixtureError::Canonicalize(_) => {
                tracing::error!(error = %error, "fixture operation failed");
                Self::internal_error(error.to_string(), None)
            },
        }
    }
}
