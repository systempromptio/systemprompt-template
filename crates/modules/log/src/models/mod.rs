use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warn => write!(f, "WARN"),
            Self::Info => write!(f, "INFO"),
            Self::Debug => write!(f, "DEBUG"),
            Self::Trace => write!(f, "TRACE"),
        }
    }
}

impl LogLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = LoggingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ERROR" => Ok(Self::Error),
            "WARN" => Ok(Self::Warn),
            "INFO" => Ok(Self::Info),
            "DEBUG" => Ok(Self::Debug),
            "TRACE" => Ok(Self::Trace),
            _ => Err(LoggingError::invalid_log_level(s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
    pub user_id: systemprompt_identifiers::UserId,
    pub session_id: systemprompt_identifiers::SessionId,
    pub task_id: Option<systemprompt_identifiers::TaskId>,
    pub trace_id: systemprompt_identifiers::TraceId,
    pub context_id: Option<systemprompt_identifiers::ContextId>,
    pub client_id: Option<systemprompt_identifiers::ClientId>,
}

impl LogEntry {
    pub fn new(level: LogLevel, module: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            module: module.into(),
            message: message.into(),
            metadata: None,
            user_id: systemprompt_identifiers::UserId::system(),
            session_id: systemprompt_identifiers::SessionId::system(),
            task_id: None,
            trace_id: systemprompt_identifiers::TraceId::system(),
            context_id: None,
            client_id: None,
        }
    }

    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    #[must_use]
    pub fn with_user_id(mut self, user_id: systemprompt_identifiers::UserId) -> Self {
        self.user_id = user_id;
        self
    }

    #[must_use]
    pub fn with_session_id(mut self, session_id: systemprompt_identifiers::SessionId) -> Self {
        self.session_id = session_id;
        self
    }

    #[must_use]
    pub fn with_task_id(mut self, task_id: systemprompt_identifiers::TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    #[must_use]
    pub fn with_trace_id(mut self, trace_id: systemprompt_identifiers::TraceId) -> Self {
        self.trace_id = trace_id;
        self
    }

    #[must_use]
    pub fn with_context_id(mut self, context_id: systemprompt_identifiers::ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }

    #[must_use]
    pub fn with_client_id(mut self, client_id: systemprompt_identifiers::ClientId) -> Self {
        self.client_id = Some(client_id);
        self
    }
}

impl LogEntry {
    pub fn validate(&self) -> Result<(), LoggingError> {
        if self.module.is_empty() {
            return Err(LoggingError::EmptyModuleName);
        }
        if self.message.is_empty() {
            return Err(LoggingError::EmptyMessage);
        }
        if let Some(metadata) = &self.metadata {
            if !metadata.is_object()
                && !metadata.is_array()
                && !metadata.is_string()
                && !metadata.is_null()
            {
                return Err(LoggingError::InvalidMetadata);
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level_str = match self.level {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN ",
            LogLevel::Info => "INFO ",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        };

        let timestamp_str = self.timestamp.format("%H:%M:%S");

        if let Some(metadata) = &self.metadata {
            write!(
                f,
                "{} [{}] {}: {} {}",
                timestamp_str,
                level_str,
                self.module,
                self.message,
                serde_json::to_string(metadata).unwrap_or_default()
            )
        } else {
            write!(
                f,
                "{} [{}] {}: {}",
                timestamp_str, level_str, self.module, self.message
            )
        }
    }
}

#[derive(Debug, FromRow)]
pub struct LogRow {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub module: String,
    pub message: String,
    pub metadata: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub trace_id: Option<String>,
    pub context_id: Option<String>,
    pub client_id: Option<String>,
}

impl LogRow {
    pub fn from_json_row(row: &systemprompt_core_database::JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let timestamp = row
            .get("timestamp")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid timestamp"))?;

        let level = row
            .get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing level"))?
            .to_string();

        let module = row
            .get("module")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing module"))?
            .to_string();

        let message = row
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing message"))?
            .to_string();

        let metadata = row
            .get("metadata")
            .and_then(|v| v.as_str())
            .map(String::from);

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let session_id = row
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let task_id = row
            .get("task_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let trace_id = row
            .get("trace_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let context_id = row
            .get("context_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let client_id = row
            .get("client_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(Self {
            id,
            timestamp,
            level,
            module,
            message,
            metadata,
            user_id,
            session_id,
            task_id,
            trace_id,
            context_id,
            client_id,
        })
    }
}

impl From<LogRow> for LogEntry {
    fn from(row: LogRow) -> Self {
        let level = row.level.parse().unwrap_or_else(|_| {
            eprintln!(
                "WARN: Invalid log level '{}' in log entry {}, defaulting to INFO",
                row.level, row.id
            );
            LogLevel::Info
        });

        Self {
            id: row.id,
            timestamp: row.timestamp,
            level,
            module: row.module,
            message: row.message,
            metadata: row.metadata.and_then(|s| serde_json::from_str(&s).ok()),
            user_id: row
                .user_id
                .map_or_else(systemprompt_identifiers::UserId::system, systemprompt_identifiers::UserId::new),
            session_id: row
                .session_id
                .map_or_else(systemprompt_identifiers::SessionId::system, systemprompt_identifiers::SessionId::new),
            task_id: row.task_id.map(systemprompt_identifiers::TaskId::new),
            trace_id: row
                .trace_id
                .map_or_else(systemprompt_identifiers::TraceId::system, systemprompt_identifiers::TraceId::new),
            context_id: row.context_id.map(systemprompt_identifiers::ContextId::new),
            client_id: row.client_id.map(systemprompt_identifiers::ClientId::new),
        }
    }
}
