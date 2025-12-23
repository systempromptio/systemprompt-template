use serde_json::{json, Value as JsonValue};
use systemprompt_models::artifacts::{DashboardArtifact, ToolResponse};

#[must_use] pub fn operations_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "required": ["action"],
        "properties": {
            "action": {
                "type": "string",
                "enum": ["list_files", "delete_file", "delete_content", "validate_skills", "validate_agents", "validate_config"],
                "description": "Operation to perform: list_files, delete_file, delete_content, validate_skills, validate_agents, or validate_config"
            },
            "uuid": {
                "type": "string",
                "description": "UUID of the resource to delete (required for delete_file and delete_content)"
            },
            "limit": {
                "type": "integer",
                "description": "Maximum number of files to return for list_files (default: 100)",
                "default": 100
            },
            "offset": {
                "type": "integer",
                "description": "Number of files to skip for pagination in list_files (default: 0)",
                "default": 0
            }
        }
    })
}

#[must_use] pub fn operations_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}
