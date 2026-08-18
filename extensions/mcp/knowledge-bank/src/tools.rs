//! Tool definitions exposed by the `knowledge-bank` MCP server.
//!
//! Three tools mirror the contract Astound's real RAG server must satisfy:
//! `search_project_context` and `list_documents` for any signed-in user, and
//! `upload_document` restricted to admins (enforced in `server::tool`).

use rmcp::model::{MetaObject, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{McpOutputSchema, default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::CliArtifact;

pub const SERVER_NAME: &str = "knowledge-bank";
pub const TOOL_SEARCH: &str = "search_project_context";
pub const TOOL_LIST: &str = "list_documents";
pub const TOOL_UPLOAD: &str = "upload_document";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchInput {
    /// Free-text query over project context (transcripts, Jira tickets,
    /// Confluence pages).
    pub query: String,
    /// Optional project id to scope the search (e.g. "acme-storefront").
    pub project_id: Option<String>,
    /// Maximum number of documents to return (default 5).
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListInput {
    /// Optional project id to scope the listing.
    pub project_id: Option<String>,
    /// Optional document type filter: "transcript", "jira", or "confluence".
    pub doc_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UploadInput {
    /// Document type: "transcript", "jira", or "confluence".
    pub doc_type: String,
    /// Project id the document belongs to.
    pub project_id: String,
    /// Human-readable document title.
    pub title: String,
    /// Full document text.
    pub content: String,
}

struct ToolDef<'a> {
    name: &'a str,
    title: &'a str,
    description: &'a str,
    // JSON: protocol boundary
    input_schema: serde_json::Value,
}

fn create_tool(def: &ToolDef<'_>) -> Tool {
    let input_obj = def
        .input_schema
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);
    let output_obj = <CliArtifact as McpOutputSchema>::validated_schema()
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);

    let mut tool = Tool::default();
    tool.name = def.name.to_owned().into();
    tool.title = Some(def.title.to_owned());
    tool.description = Some(def.description.to_owned().into());
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
    vec![
        create_tool(&ToolDef {
            name: TOOL_SEARCH,
            title: "Search Project Context",
            description: "Search the project knowledge bank (workshop transcripts, Jira tickets, \
                          Confluence pages) for prior decisions and context. Use this before \
                          proposing an approach — prior project decisions outrank general best \
                          practice.",
            input_schema: schemars::schema_for!(SearchInput).to_value(),
        }),
        create_tool(&ToolDef {
            name: TOOL_LIST,
            title: "List Knowledge Bank Documents",
            description: "List documents in the project knowledge bank, optionally filtered by \
                          project and document type (transcript, jira, confluence).",
            input_schema: schemars::schema_for!(ListInput).to_value(),
        }),
        create_tool(&ToolDef {
            name: TOOL_UPLOAD,
            title: "Upload Document",
            description: "Upload a document (meeting transcript, ticket summary, or page) to the \
                          project knowledge bank. Admin role required.",
            input_schema: schemars::schema_for!(UploadInput).to_value(),
        }),
    ]
}
