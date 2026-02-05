use rmcp::model::{Meta, Tool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::{ListArtifact, TextArtifact, ToolResponse};

pub const SERVER_NAME: &str = "soul";

fn text_output_schema() -> serde_json::Map<String, serde_json::Value> {
    ToolResponse::<TextArtifact>::schema()
        .as_object()
        .cloned()
        .expect("schema must be object")
}

fn list_output_schema() -> serde_json::Map<String, serde_json::Value> {
    ToolResponse::<ListArtifact>::schema()
        .as_object()
        .cloned()
        .expect("schema must be object")
}

fn create_ui_meta() -> Meta {
    Meta(tool_ui_meta(SERVER_NAME, &default_tool_visibility()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetContextInput {
    #[serde(default)]
    pub memory_types: Option<Vec<String>>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default = "default_max_items")]
    pub max_items: i64,
}

fn default_max_items() -> i64 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMemoryInput {
    pub memory_type: String,
    pub category: String,
    pub subject: String,
    pub content: String,
    #[serde(default)]
    pub context_text: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    pub query: String,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgetInput {
    pub id: String,
}

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    vec![
        create_get_context_tool(),
        create_store_tool(),
        create_search_tool(),
        create_forget_tool(),
    ]
}

fn create_get_context_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "memory_types": {
                "type": "array",
                "items": { "type": "string", "enum": ["core", "long_term", "short_term", "working"] },
                "description": "Filter by memory types. Default: all types."
            },
            "subject": {
                "type": "string",
                "description": "Filter memories by subject (partial match)."
            },
            "max_items": {
                "type": "integer",
                "default": 50,
                "description": "Maximum number of memories to return."
            }
        }
    });

    Tool {
        name: "memory_get_context".to_string().into(),
        title: Some("Get Memory Context".to_string()),
        description: Some(
            "Retrieve formatted memory context for injection into prompts. \
             Returns memories ordered by type (core > long_term > short_term > working) and priority."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(text_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

fn create_store_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "memory_type": {
                "type": "string",
                "enum": ["core", "long_term", "short_term", "working"],
                "description": "Memory type: core (permanent), long_term (persistent), short_term (24-48h), working (session)"
            },
            "category": {
                "type": "string",
                "enum": ["fact", "preference", "goal", "relationship", "reminder"],
                "description": "Memory category"
            },
            "subject": {
                "type": "string",
                "description": "Who/what this memory is about (e.g., 'user', 'project', person name)"
            },
            "content": {
                "type": "string",
                "description": "The actual memory content"
            },
            "context_text": {
                "type": "string",
                "description": "Optional pre-formatted text for context injection"
            },
            "priority": {
                "type": "integer",
                "minimum": 0,
                "maximum": 100,
                "default": 50,
                "description": "Priority for ordering in context (0-100, higher = more important)"
            },
            "tags": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Searchable tags"
            }
        },
        "required": ["memory_type", "category", "subject", "content"]
    });

    Tool {
        name: "memory_store".to_string().into(),
        title: Some("Store Memory".to_string()),
        description: Some(
            "Store a new memory in the database. Memories are classified by type and category \
             for organized retrieval and context injection."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(text_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

fn create_search_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query (matches content, subject, and tags)"
            },
            "memory_type": {
                "type": "string",
                "enum": ["core", "long_term", "short_term", "working"],
                "description": "Optional filter by memory type"
            },
            "category": {
                "type": "string",
                "enum": ["fact", "preference", "goal", "relationship", "reminder"],
                "description": "Optional filter by category"
            },
            "limit": {
                "type": "integer",
                "default": 20,
                "description": "Maximum results to return"
            }
        },
        "required": ["query"]
    });

    Tool {
        name: "memory_search".to_string().into(),
        title: Some("Search Memories".to_string()),
        description: Some(
            "Search memories by content, subject, or tags. Returns matching memories \
             ordered by priority and recency."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(list_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

fn create_forget_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "id": {
                "type": "string",
                "description": "The memory ID to forget (soft-delete)"
            }
        },
        "required": ["id"]
    });

    Tool {
        name: "memory_forget".to_string().into(),
        title: Some("Forget Memory".to_string()),
        description: Some(
            "Soft-delete a memory by ID. The memory is marked as inactive but not permanently deleted."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(text_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}
