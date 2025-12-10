use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{LogLevel, LoggingError};

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
