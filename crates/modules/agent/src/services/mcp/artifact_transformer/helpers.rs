use serde_json::Value as JsonValue;
use systemprompt_models::artifacts::types::ArtifactType;

pub fn unwrap_tool_response(structured_content: &JsonValue) -> (&JsonValue, Option<&JsonValue>) {
    if let (Some(artifact), Some(metadata)) = (
        structured_content.get("artifact"),
        structured_content.get("_metadata"),
    ) {
        (artifact, Some(metadata))
    } else {
        (structured_content, None)
    }
}

pub fn extract_artifact_id(structured_content: &JsonValue) -> Option<String> {
    structured_content
        .get("artifact_id")
        .and_then(|v| v.as_str())
        .map(String::from)
}

pub fn extract_skill_id(structured_content: &JsonValue) -> Option<String> {
    structured_content
        .get("skill_id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            structured_content
                .get("_metadata")
                .and_then(|m| m.get("skill_id"))
                .and_then(|v| v.as_str())
                .map(String::from)
        })
}

pub fn extract_execution_id(structured_content: &JsonValue) -> Option<String> {
    structured_content
        .get("_metadata")
        .and_then(|m| m.get("execution_id"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

pub fn artifact_type_to_string(artifact_type: &ArtifactType) -> String {
    match artifact_type {
        ArtifactType::Text => "text".to_string(),
        ArtifactType::Table => "table".to_string(),
        ArtifactType::Chart => "chart".to_string(),
        ArtifactType::Form => "form".to_string(),
        ArtifactType::Dashboard => "dashboard".to_string(),
        ArtifactType::PresentationCard => "presentation_card".to_string(),
        ArtifactType::List => "list".to_string(),
        ArtifactType::CopyPasteText => "copy_paste_text".to_string(),
        ArtifactType::Blog => "blog".to_string(),
    }
}

pub fn calculate_fingerprint(tool_name: &str, tool_arguments: Option<&JsonValue>) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let args_str = tool_arguments
        .and_then(|args| serde_json::to_string(args).ok())
        .unwrap_or_default();

    let mut hasher = DefaultHasher::new();
    args_str.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{}-{:x}", tool_name, hash)
}
