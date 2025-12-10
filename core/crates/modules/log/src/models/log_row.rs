use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use super::{LogEntry, LogLevel};

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
        let level = row.level.parse().unwrap_or(LogLevel::Info);

        Self {
            id: row.id,
            timestamp: row.timestamp,
            level,
            module: row.module,
            message: row.message,
            metadata: row.metadata.and_then(|s| serde_json::from_str(&s).ok()),
            user_id: row.user_id.map_or_else(
                systemprompt_identifiers::UserId::system,
                systemprompt_identifiers::UserId::new,
            ),
            session_id: row.session_id.map_or_else(
                systemprompt_identifiers::SessionId::system,
                systemprompt_identifiers::SessionId::new,
            ),
            task_id: row.task_id.map(systemprompt_identifiers::TaskId::new),
            trace_id: row.trace_id.map_or_else(
                systemprompt_identifiers::TraceId::system,
                systemprompt_identifiers::TraceId::new,
            ),
            context_id: row.context_id.map(systemprompt_identifiers::ContextId::new),
            client_id: row.client_id.map(systemprompt_identifiers::ClientId::new),
        }
    }
}
