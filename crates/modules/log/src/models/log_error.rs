#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Invalid log entry: {field} {reason}")]
    InvalidLogEntry { field: String, reason: String },

    #[error("Log entry validation failed: {message}")]
    ValidationError { message: String },

    #[error("Invalid log level: {level}")]
    InvalidLogLevel { level: String },

    #[error("Log entry not found: {id}")]
    LogEntryNotFound { id: String },

    #[error("Empty log module name")]
    EmptyModuleName,

    #[error("Empty log message")]
    EmptyMessage,

    #[error("Invalid metadata format")]
    InvalidMetadata,

    #[error("Database operation failed")]
    DatabaseError(#[from] sqlx::Error),

    #[error("JSON serialization failed")]
    JsonError(#[from] serde_json::Error),

    #[error("UUID generation failed")]
    UuidError(#[from] uuid::Error),

    #[error("DateTime parsing failed")]
    DateTimeError(#[from] chrono::ParseError),

    #[error("Log repository operation failed: {operation}")]
    RepositoryError { operation: String },

    #[error("Cleanup operation failed: deleted {count} entries")]
    CleanupError { count: u64 },

    #[error("Pagination parameters invalid: page={page}, per_page={per_page}")]
    PaginationError { page: i32, per_page: i32 },

    #[error("Log filter invalid: {filter_type}={value}")]
    FilterError { filter_type: String, value: String },

    #[error("Terminal output failed")]
    TerminalError,

    #[error("Database connection not available")]
    DatabaseUnavailable,

    #[error("Query operation failed")]
    QueryError(#[from] anyhow::Error),
}

impl LoggingError {
    pub fn invalid_log_entry(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidLogEntry {
            field: field.into(),
            reason: reason.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    pub fn invalid_log_level(level: impl Into<String>) -> Self {
        Self::InvalidLogLevel {
            level: level.into(),
        }
    }

    pub fn log_entry_not_found(id: impl Into<String>) -> Self {
        Self::LogEntryNotFound { id: id.into() }
    }

    pub fn repository_error(operation: impl Into<String>) -> Self {
        Self::RepositoryError {
            operation: operation.into(),
        }
    }

    pub const fn cleanup_error(count: u64) -> Self {
        Self::CleanupError { count }
    }

    pub const fn pagination_error(page: i32, per_page: i32) -> Self {
        Self::PaginationError { page, per_page }
    }

    pub fn filter_error(filter_type: impl Into<String>, value: impl Into<String>) -> Self {
        Self::FilterError {
            filter_type: filter_type.into(),
            value: value.into(),
        }
    }

    pub fn into_sqlx_error(self) -> sqlx::Error {
        sqlx::Error::Protocol(format!("{self}"))
    }
}
