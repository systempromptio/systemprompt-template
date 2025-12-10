use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task UUID missing from database row")]
    MissingTaskUuid,

    #[error("Agent name not found for task {task_id}")]
    MissingAgentName { task_id: String },

    #[error("Context ID missing from database row")]
    MissingContextId,

    #[error("Invalid task state: {state}")]
    InvalidTaskState { state: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid datetime for field '{field}'")]
    InvalidDatetime { field: String },

    #[error("JSON parse error for field '{field}': {source}")]
    JsonParse {
        field: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Metadata parse error: {0}")]
    InvalidMetadata(#[from] serde_json::Error),

    #[error("Empty task ID provided")]
    EmptyTaskId,

    #[error("Invalid task ID format: {id}")]
    InvalidTaskIdFormat { id: String },

    #[error("Message ID missing from database row")]
    MissingMessageId,

    #[error("Tool name missing for tool execution")]
    MissingToolName,

    #[error("Tool call ID missing for tool execution")]
    MissingCallId,

    #[error("Created timestamp missing from database")]
    MissingCreatedTimestamp,

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Context UUID missing from database row")]
    MissingUuid,

    #[error("Context name missing from database row")]
    MissingName,

    #[error("User ID missing from database row")]
    MissingUserId,

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid datetime for field '{field}'")]
    InvalidDatetime { field: String },

    #[error("JSON parse error for field '{field}': {source}")]
    JsonParse {
        field: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Role serialization error: {0}")]
    RoleSerialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("Artifact UUID missing from database row")]
    MissingUuid,

    #[error("Artifact type missing from database row")]
    MissingType,

    #[error("Context ID missing for artifact")]
    MissingContextId,

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid datetime for field '{field}'")]
    InvalidDatetime { field: String },

    #[error("JSON parse error for field '{field}': {source}")]
    JsonParse {
        field: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Metadata parse error: {0}")]
    InvalidMetadata(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("Metadata validation error: {0}")]
    MetadataValidation(String),
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Tool name missing in tool call")]
    MissingToolName,

    #[error("Tool result error flag is required but was not provided")]
    MissingErrorFlag,

    #[error("Message ID missing")]
    MissingMessageId,

    #[error("Request ID missing")]
    MissingRequestId,

    #[error("Latency value missing or invalid")]
    InvalidLatency,

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Task error: {0}")]
    Task(#[from] TaskError),

    #[error("Context error: {0}")]
    Context(#[from] ContextError),

    #[error("Artifact error: {0}")]
    Artifact(#[from] ArtifactError),

    #[error("A2A protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Repository error: {0}")]
    Repository(#[from] anyhow::Error),

    #[error("Database error: {0}")]
    Database(String),
}
