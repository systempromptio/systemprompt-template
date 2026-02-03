use serde_json::json;
use systemprompt::models::artifacts::{ResearchArtifact, ToolResponse};

#[must_use]
pub fn input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "topic": {
                "type": "string",
                "description": "The topic to research"
            },
            "skill_id": {
                "type": "string",
                "description": "Must be 'research_blog'"
            },
            "focus_areas": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional specific areas to focus research on"
            }
        },
        "required": ["topic", "skill_id"]
    })
}

#[must_use]
pub fn output_schema() -> serde_json::Value {
    ToolResponse::<ResearchArtifact>::schema()
}

pub fn extract_string_array(
    args: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
