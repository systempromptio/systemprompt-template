use crate::prompts::TemplatePrompts;
use anyhow::Result;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

/// ARTIFACT SUPPORT PATTERN
///
/// To add artifact rendering to your MCP tools:
///
/// 1. Define output schema helper functions:
/// ```rust
/// pub fn my_card_output_schema() -> serde_json::Value {
///     json!({
///         "type": "object",
///         "x-artifact-type": "presentation_card",  // ← REQUIRED for artifacts
///         "x-presentation-hints": {"theme": "gradient"},
///         "properties": {
///             "title": {"type": "string"},
///             "subtitle": {"type": "string"},
///             "sections": {"type": "array", "items": {"type": "object"}}
///         }
///     })
/// }
/// ```
///
/// 2. Add output_schema to Tool definition:
/// ```rust
/// Tool {
///     name: "my_tool".into(),
///     output_schema: Some(Arc::new(my_card_output_schema().as_object().unwrap().clone())),
///     // ... other fields
/// }
/// ```
///
/// 3. Update server.rs get_output_schema_for_tool() to map tool name to schema
///
/// Supported Artifact Types:
/// - "presentation_card" - Interactive cards with CTAs
/// - "table" - Tabular data
/// - "chart" - Data visualization
/// - "code" - Code snippets
/// - "markdown" - Rich text

#[derive(Debug, Clone)]
pub struct TemplateTools {
    _db_pool: DbPool,
    _prompts: Arc<TemplatePrompts>,
}

impl TemplateTools {
    pub fn new(db_pool: DbPool, prompts: Arc<TemplatePrompts>) -> Self {
        Self {
            _db_pool: db_pool,
            _prompts: prompts,
        }
    }

    pub async fn list_tools(&self) -> Result<ListToolsResult, McpError> {
        let tools = vec![];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    pub async fn call_tool(
        &self,
        name: &str,
        _request: CallToolRequestParam,
        _ctx: RequestContext<RoleServer>,
        _logger: systemprompt_core_logging::LogService,
    ) -> Result<CallToolResult, McpError> {
        match name {
            _ => Err(McpError::method_not_found::<CallToolRequestMethod>()),
        }
    }
}
