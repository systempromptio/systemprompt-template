use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const SERVER_NAME: &str = "soul";

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

fn get_context_input_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), serde_json::json!("object"));
    map.insert("properties".to_string(), serde_json::json!({
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
    }));
    map
}

fn create_get_context_tool() -> Tool {
    Tool {
        name: "memory_get_context".to_string().into(),
        title: Some("Get Memory Context".to_string()),
        description: Some(
            "Retrieve formatted memory context for injection into prompts. \
             Returns memories ordered by type (core > long_term > short_term > working) and priority."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(get_context_input_schema()),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    }
}

fn store_input_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), serde_json::json!("object"));
    map.insert("properties".to_string(), serde_json::json!({
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
    }));
    map.insert(
        "required".to_string(),
        serde_json::json!(["memory_type", "category", "subject", "content"]),
    );
    map
}

fn create_store_tool() -> Tool {
    Tool {
        name: "memory_store".to_string().into(),
        title: Some("Store Memory".to_string()),
        description: Some(
            "Store a new memory in the database. Memories are classified by type and category \
             for organized retrieval and context injection."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(store_input_schema()),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    }
}

fn search_input_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), serde_json::json!("object"));
    map.insert(
        "properties".to_string(),
        serde_json::json!({
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
        }),
    );
    map.insert("required".to_string(), serde_json::json!(["query"]));
    map
}

fn create_search_tool() -> Tool {
    Tool {
        name: "memory_search".to_string().into(),
        title: Some("Search Memories".to_string()),
        description: Some(
            "Search memories by content, subject, or tags. Returns matching memories \
             ordered by priority and recency."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(search_input_schema()),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    }
}

fn forget_input_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), serde_json::json!("object"));
    map.insert(
        "properties".to_string(),
        serde_json::json!({
            "id": {
                "type": "string",
                "description": "The memory ID to forget (soft-delete)"
            }
        }),
    );
    map.insert("required".to_string(), serde_json::json!(["id"]));
    map
}

fn create_forget_tool() -> Tool {
    Tool {
        name: "memory_forget".to_string().into(),
        title: Some("Forget Memory".to_string()),
        description: Some(
            "Soft-delete a memory by ID. The memory is marked as inactive but not permanently deleted."
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(forget_input_schema()),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    }
}
