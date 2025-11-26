use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Invalid roles JSON for user {user_id}: {source}")]
    InvalidRoles {
        user_id: String,
        source: serde_json::Error,
    },

    #[error("Invalid user type: {user_type}")]
    InvalidUserType { user_type: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}
