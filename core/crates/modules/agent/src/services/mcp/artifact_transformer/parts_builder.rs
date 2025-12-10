use crate::error::ArtifactError;
use crate::models::a2a::message::{DataPart, FilePart, FileWithBytes, Part, TextPart};
use rmcp::model::{CallToolResult, RawContent};
use serde_json::Value as JsonValue;

use super::helpers::unwrap_tool_response;

pub fn build_parts_from_result(tool_result: &CallToolResult) -> Result<Vec<Part>, ArtifactError> {
    let mut parts = Vec::new();

    if let Some(structured) = &tool_result.structured_content {
        let (actual_data, _) = unwrap_tool_response(structured);

        if let Some(obj) = actual_data.as_object() {
            let cleaned_data = obj.clone();
            parts.push(Part::Data(DataPart { data: cleaned_data }));
            return Ok(parts);
        }
    }

    for content_item in &tool_result.content {
        match &content_item.raw {
            RawContent::Text(text_content) => {
                parts.push(Part::Text(TextPart {
                    text: text_content.text.clone(),
                }));
            },
            RawContent::Image(image_content) => {
                parts.push(Part::File(FilePart {
                    file: FileWithBytes {
                        bytes: image_content.data.clone(),
                        mime_type: Some(image_content.mime_type.clone()),
                        name: None,
                    },
                }));
            },
            RawContent::ResourceLink(resource) => {
                parts.push(Part::File(FilePart {
                    file: FileWithBytes {
                        name: Some(resource.uri.clone()),
                        mime_type: resource.mime_type.clone(),
                        bytes: String::new(),
                    },
                }));
            },
            _ => {},
        }
    }

    if !parts.is_empty() {
        return Ok(parts);
    }

    Err(ArtifactError::Transform(format!(
        "Tool result must have either 'structured_content' or 'content' array. Received \
         CallToolResult with {} content items and structured_content: {}",
        tool_result.content.len(),
        tool_result.structured_content.is_some()
    )))
}

pub fn build_parts(tool_result: &JsonValue) -> Result<Vec<Part>, ArtifactError> {
    let mut parts = Vec::new();

    let (actual_data, _) = unwrap_tool_response(tool_result);

    if let Some(structured) = tool_result.get("structured_content") {
        let (unwrapped, _) = unwrap_tool_response(structured);
        let obj = unwrapped.as_object().ok_or_else(|| {
            ArtifactError::Transform(format!(
                "'structured_content' must be an object, got: {}",
                serde_json::to_string_pretty(structured)
                    .unwrap_or_else(|_| "invalid JSON".to_string())
            ))
        })?;

        parts.push(Part::Data(DataPart { data: obj.clone() }));
        return Ok(parts);
    }

    if let Some(obj) = actual_data.as_object() {
        parts.push(Part::Data(DataPart { data: obj.clone() }));
        return Ok(parts);
    }

    if let Some(content) = tool_result.get("content") {
        if let Some(arr) = content.as_array() {
            for item in arr {
                if let Some(content_type) = item.get("type").and_then(|t| t.as_str()) {
                    match content_type {
                        "text" => {
                            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                parts.push(Part::Text(TextPart {
                                    text: text.to_string(),
                                }));
                            }
                        },
                        "image" => {
                            if let Some(data) = item.get("data").and_then(|d| d.as_str()) {
                                let mime_type = item
                                    .get("mimeType")
                                    .and_then(|m| m.as_str())
                                    .map(|s| s.to_string());

                                parts.push(Part::File(FilePart {
                                    file: FileWithBytes {
                                        bytes: data.to_string(),
                                        mime_type,
                                        name: None,
                                    },
                                }));
                            }
                        },
                        "resource" => {
                            if let Some(uri) = item.get("uri").and_then(|u| u.as_str()) {
                                let mime_type = item
                                    .get("mimeType")
                                    .and_then(|m| m.as_str())
                                    .map(|s| s.to_string());

                                parts.push(Part::File(FilePart {
                                    file: FileWithBytes {
                                        name: Some(uri.to_string()),
                                        mime_type,
                                        bytes: String::new(),
                                    },
                                }));
                            }
                        },
                        _ => {},
                    }
                }
            }
            if !parts.is_empty() {
                return Ok(parts);
            }
        }
    }

    Err(ArtifactError::Transform(format!(
        "Tool result must have either 'structured_content' or 'content' array. Received: {}",
        serde_json::to_string_pretty(tool_result).unwrap_or_else(|_| "invalid JSON".to_string())
    )))
}
