//! Tool definitions exposed by the knowledge-bank MCP server.
//!
//! The contract mirrors the Node project-context MCP server being migrated
//! here (`search`, `list_sources`, `index_stats`), renamed into this
//! instance's namespace: `search_knowledge`, `list_knowledge_sources`,
//! `knowledge_index_stats`.

use rmcp::model::{MetaObject, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{McpOutputSchema, default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::CliArtifact;

pub const SERVER_NAME: &str = "knowledge-bank";
pub const SEARCH_TOOL: &str = "search_knowledge";
pub const LIST_SOURCES_TOOL: &str = "list_knowledge_sources";
pub const INDEX_STATS_TOOL: &str = "knowledge_index_stats";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchKnowledgeInput {
    #[schemars(description = "Natural-language query to search the knowledge bank with")]
    pub query: String,
    #[schemars(
        description = "Source categories to search (ids from list_knowledge_sources, e.g. \
                       meeting_notes, confluence, jira). Live categories must be listed \
                       explicitly to search broadly; archive categories are searched only when \
                       named here. Optional while no backend is configured"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_types: Option<Vec<String>>,
    #[schemars(
        range(min = 1, max = 20),
        description = "Maximum results to return (1-20). Omit for the configured default"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[expect(
    clippy::empty_structs_with_brackets,
    reason = "MCP tool arguments arrive as an empty JSON object `{}`; a unit struct would fail \
              to deserialize from it"
)]
pub struct ListKnowledgeSourcesInput {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[expect(
    clippy::empty_structs_with_brackets,
    reason = "MCP tool arguments arrive as an empty JSON object `{}`; a unit struct would fail \
              to deserialize from it"
)]
pub struct IndexStatsInput {}

#[must_use]
// JSON: protocol boundary
pub fn search_input_schema() -> serde_json::Value {
    schemars::schema_for!(SearchKnowledgeInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn list_sources_input_schema() -> serde_json::Value {
    schemars::schema_for!(ListKnowledgeSourcesInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn index_stats_input_schema() -> serde_json::Value {
    schemars::schema_for!(IndexStatsInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn output_schema() -> serde_json::Value {
    <CliArtifact as McpOutputSchema>::validated_schema()
}

// JSON: protocol boundary
fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: &serde_json::Value,
    output: &serde_json::Value,
) -> Tool {
    let input_obj = input_schema
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);
    let output_obj = output
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);

    let mut tool = Tool::default();
    tool.name = name.to_owned().into();
    tool.title = Some(title.to_owned());
    tool.description = Some(description.to_owned().into());
    tool.input_schema = Arc::new(input_obj);
    tool.output_schema = Some(Arc::new(output_obj));
    tool.meta = Some(MetaObject(tool_ui_meta(
        SERVER_NAME,
        &default_tool_visibility(),
    )));
    tool
}

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    let output = output_schema();
    vec![
        create_tool(
            SEARCH_TOOL,
            "Search Knowledge",
            "Search the enterprise knowledge bank. Pass a natural-language query; list the \
             source categories to search in source_types (get ids from list_knowledge_sources; \
             archive categories are searched only when named); cap results with top_k (1-20).\n\n\
             Example: {\"query\": \"deployment runbook for the gateway\", \"source_types\": \
             [\"confluence\", \"meeting_notes\"], \"top_k\": 5}",
            &search_input_schema(),
            &output,
        ),
        create_tool(
            LIST_SOURCES_TOOL,
            "List Knowledge Sources",
            "List the knowledge source categories this instance can search, with document counts \
             and last sync time where known. Takes no arguments.",
            &list_sources_input_schema(),
            &output,
        ),
        create_tool(
            INDEX_STATS_TOOL,
            "Knowledge Index Stats",
            "Report retrieval-index statistics: document and chunk totals, last build time, and \
             the index version stamp. Takes no arguments.",
            &index_stats_input_schema(),
            &output,
        ),
    ]
}
