use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use systemprompt_identifiers::McpExecutionId;

/// Strongly-typed metadata for MCP tool execution results.
/// Serialized into `CallToolResult`._meta field per MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpToolResultMetadata {
    /// REQUIRED: MCP execution ID from `mcp_tool_executions` table
    pub mcp_execution_id: McpExecutionId,

    /// Optional: Execution timing in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<i64>,

    /// Optional: Server version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
}

impl McpToolResultMetadata {
    /// Create new metadata with required `execution_id`
    pub const fn new(mcp_execution_id: McpExecutionId) -> Self {
        Self {
            mcp_execution_id,
            execution_time_ms: None,
            server_version: None,
        }
    }

    /// Builder: Add execution time
    pub const fn with_execution_time(mut self, time_ms: i64) -> Self {
        self.execution_time_ms = Some(time_ms);
        self
    }

    /// Builder: Add server version
    pub fn with_server_version(mut self, version: impl Into<String>) -> Self {
        self.server_version = Some(version.into());
        self
    }

    /// Validate that required fields are present and valid
    pub fn validate(&self) -> Result<()> {
        if self.mcp_execution_id.as_str().is_empty() {
            return Err(anyhow!(
                "McpToolResultMetadata: mcp_execution_id cannot be empty"
            ));
        }
        Ok(())
    }

    /// Convert to `rmcp::model::Meta` (protocol type)
    /// Validates before conversion, returns error if invalid
    pub fn to_meta(&self) -> Result<rmcp::model::Meta> {
        self.validate()?;

        let json_value = serde_json::to_value(self)?;
        let json_object = json_value
            .as_object()
            .ok_or_else(|| anyhow!("Failed to serialize McpToolResultMetadata as JSON object"))?
            .clone();

        Ok(rmcp::model::Meta(json_object))
    }

    /// Parse from `rmcp::model::Meta` (protocol type) with validation
    /// Fails fast if metadata is missing or invalid
    pub fn from_meta(meta: &rmcp::model::Meta) -> Result<Self> {
        let json_value = Value::Object(meta.0.clone());
        let metadata: Self = serde_json::from_value(json_value)
            .map_err(|e| anyhow!("Failed to parse McpToolResultMetadata from _meta: {e}. Expected fields: mcp_execution_id (required), execution_time_ms (optional), server_version (optional)"))?;

        metadata.validate()?;
        Ok(metadata)
    }

    /// Try to extract from `CallToolResult`._meta, fail fast if invalid
    pub fn from_call_tool_result(result: &rmcp::model::CallToolResult) -> Result<Self> {
        let meta = result.meta.as_ref().ok_or_else(|| {
            anyhow!("CallToolResult._meta is missing (required for MCP execution tracking)")
        })?;

        Self::from_meta(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate() {
        let mcp_execution_id = McpExecutionId::generate();
        let metadata = McpToolResultMetadata::new(mcp_execution_id);
        assert!(metadata.validate().is_ok());
    }

    #[test]
    fn test_to_meta_and_back() {
        let mcp_execution_id = McpExecutionId::generate();
        let original = McpToolResultMetadata::new(mcp_execution_id)
            .with_execution_time(150)
            .with_server_version("1.0.0");

        let meta = original.to_meta().unwrap();
        let parsed = McpToolResultMetadata::from_meta(&meta).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_missing_meta_fails() {
        let result = rmcp::model::CallToolResult {
            content: vec![],
            structured_content: None,
            is_error: None,
            meta: None,
        };

        assert!(McpToolResultMetadata::from_call_tool_result(&result).is_err());
    }
}
