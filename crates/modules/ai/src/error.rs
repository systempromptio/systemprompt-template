use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Model not specified and no default available for provider {provider}")]
    ModelNotSpecified { provider: String },

    #[error("Request metadata missing required field: {field}")]
    MissingMetadata { field: String },

    #[error("User context required for billing and audit trails")]
    MissingUserContext,

    #[error("Provider {provider} returned empty response")]
    EmptyProviderResponse { provider: String },

    #[error("Tool call schema validation failed: {reason}")]
    InvalidToolSchema { reason: String },

    #[error("Authentication required for service {service_id}")]
    AuthenticationRequired { service_id: String },

    #[error("Structured output validation failed after {retries} attempts: {details}")]
    StructuredOutputFailed { retries: usize, details: String },

    #[error("Provider {provider} error: {message}")]
    ProviderError { provider: String, message: String },

    #[error("Serialization failed: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Message history cannot be serialized to JSON")]
    MessageSerializationFailed,

    #[error("Tool {tool_name} missing required field: {field}")]
    MissingToolField { tool_name: String, field: String },

    #[error("Tool description cannot be empty for tool: {tool_name}")]
    EmptyToolDescription { tool_name: String },

    #[error("No tool calls found in provider response")]
    NoToolCalls,

    #[error("Rate limit exceeded for provider {provider}: {details}")]
    RateLimit { provider: String, details: String },

    #[error("Invalid API credentials for provider {provider}")]
    AuthenticationFailed { provider: String },

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Database operation failed: {0}")]
    DatabaseError(#[from] anyhow::Error),

    #[error("MCP service {service_id} not found or not configured")]
    McpServiceNotFound { service_id: String },

    #[error("MCP service {service_id} requires OAuth authentication but no token available")]
    McpAuthenticationMissing { service_id: String },

    #[error("Failed to determine service authentication requirements: {details}")]
    ServiceAuthCheckFailed { details: String },

    #[error("Storage operation failed: {0}")]
    StorageError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("AI request not found: {0}")]
    NotFound(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid data: {field} - {reason}")]
    InvalidData { field: String, reason: String },
}

pub type Result<T> = std::result::Result<T, AiError>;

impl AiError {
    pub fn with_context(self, context: &str) -> anyhow::Error {
        anyhow::anyhow!("{context}: {self}")
    }
}
