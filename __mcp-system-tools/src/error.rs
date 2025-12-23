use rmcp::ErrorData as McpError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Path is not a file: {path}")]
    NotAFile { path: String },

    #[error("Path is not a directory: {path}")]
    NotADirectory { path: String },

    #[error("Access denied: {path} is outside allowed roots")]
    AccessDenied { path: String },

    #[error("Path does not exist: {path} ({details})")]
    PathDoesNotExist { path: String, details: String },

    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    #[error("Missing required parameter: {name}")]
    MissingParameter { name: String },

    #[error("Invalid glob pattern: {details}")]
    InvalidGlobPattern { details: String },

    #[error("Invalid regex pattern: {details}")]
    InvalidRegexPattern { details: String },

    #[error("String not found: '{needle}' does not exist in {path}")]
    StringNotFound { needle: String, path: String },

    #[error("Ambiguous edit: found {count} occurrences of the search string")]
    AmbiguousEdit { count: usize },

    #[error("IO error: {details}")]
    IoError { details: String },

    #[error("No file roots configured")]
    NoFileRoots,
}

impl From<ToolError> for McpError {
    fn from(error: ToolError) -> Self {
        match error {
            ToolError::IoError { .. } => McpError::internal_error(error.to_string(), None),
            _ => McpError::invalid_params(error.to_string(), None),
        }
    }
}

impl From<std::io::Error> for ToolError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError {
            details: error.to_string(),
        }
    }
}
