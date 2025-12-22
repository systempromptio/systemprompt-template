mod ai_caller;
mod handler;
mod models;

pub use handler::handle;

use serde_json::json;

pub fn file_context_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "What context to gather (e.g., 'understand the authentication system', 'find how API routes are defined')"
            },
            "path": {
                "type": "string",
                "description": "Starting directory path (defaults to FILE_ROOT)"
            },
            "max_iterations": {
                "type": "integer",
                "description": "Maximum reasoning iterations (default: 5, max: 10)"
            }
        },
        "required": ["query"]
    })
}
