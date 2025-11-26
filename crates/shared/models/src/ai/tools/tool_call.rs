use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use systemprompt_identifiers::AiToolCallId;

pub const EXECUTION_CONTROL_TOOL_NAME: &str = "__execution_control";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub ai_tool_call_id: AiToolCallId,
    pub name: String,
    pub arguments: JsonValue,
}

impl ToolCall {
    pub fn is_meta_tool(&self) -> bool {
        self.name == EXECUTION_CONTROL_TOOL_NAME
    }

    pub fn is_executable(&self) -> bool {
        !self.is_meta_tool()
    }
}

pub trait ToolCallExt {
    fn filter_executable(&self) -> Vec<ToolCall>;
    fn filter_storable(&self) -> Vec<ToolCall>;
}

impl ToolCallExt for [ToolCall] {
    fn filter_executable(&self) -> Vec<ToolCall> {
        self.iter()
            .filter(|tc| tc.is_executable())
            .cloned()
            .collect()
    }

    fn filter_storable(&self) -> Vec<ToolCall> {
        self.filter_executable()
    }
}

impl ToolCallExt for Vec<ToolCall> {
    fn filter_executable(&self) -> Vec<ToolCall> {
        self.as_slice().filter_executable()
    }

    fn filter_storable(&self) -> Vec<ToolCall> {
        self.as_slice().filter_storable()
    }
}

/// MCP protocol result - this is the ONLY tool result type in the system.
/// All tool execution flows through MCP and returns this type.
///
/// This type contains:
/// - `content: Vec<Content>` - Text, images, resources returned by the tool
/// - `structured_content: Option<JsonValue>` - Rich UI data (presentation cards, tables, etc.)
/// - `is_error: Option<bool>` - Whether the execution failed
/// - `meta: Option<JsonValue>` - Additional metadata
pub use rmcp::model::CallToolResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    pub id: String,
    pub request_id: String,
    pub sequence: i32,
    pub tool_name: String,
    pub service_id: String,
    pub input: JsonValue,
    pub output: Option<JsonValue>,
    pub status: String,
    pub execution_time_ms: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ToolExecution {
    pub fn from_json_row(row: &HashMap<String, JsonValue>) -> anyhow::Result<Self> {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing id"))?
            .to_string();

        let request_id = row
            .get("request_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing request_id"))?
            .to_string();

        let sequence = row
            .get("sequence")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("Missing sequence"))
            .and_then(|i| i32::try_from(i).map_err(|_| anyhow::anyhow!("Sequence out of range")))?;

        let tool_name = row
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tool_name"))?
            .to_string();

        let service_id = row
            .get("service_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing service_id"))?
            .to_string();

        let input = row
            .get("input")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(JsonValue::Null);

        let output = row
            .get("output")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str(s).ok());

        let status = row
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing status"))?
            .to_string();

        let execution_time_ms = row
            .get("execution_time_ms")
            .and_then(serde_json::Value::as_i64)
            .and_then(|i| i32::try_from(i).ok());

        let error_message = row
            .get("error_message")
            .and_then(|v| v.as_str())
            .map(String::from);

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid created_at"))?;

        Ok(Self {
            id,
            request_id,
            sequence,
            tool_name,
            service_id,
            input,
            output,
            status,
            execution_time_ms,
            error_message,
            created_at,
        })
    }
}
