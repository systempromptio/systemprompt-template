use crate::error::ArtifactError;
use rmcp::model::CallToolResult;
use serde_json::{json, Value as JsonValue};
use systemprompt_models::artifacts::types::ArtifactType;

use super::helpers::unwrap_tool_response;

pub fn infer_type(
    tool_result: &JsonValue,
    schema: Option<&JsonValue>,
    tool_name: &str,
) -> Result<ArtifactType, ArtifactError> {
    if let Some(artifact_type) = extract_artifact_type_from_data(tool_result) {
        if let Some(parsed) = parse_artifact_type(&artifact_type) {
            return Ok(parsed);
        }
        return Err(ArtifactError::Transform(format!(
            "Tool '{}' has unknown x-artifact-type '{}'. Valid types: text, table, chart, form, \
             dashboard, presentation_card, list, copy_paste_text, blog",
            tool_name, artifact_type
        )));
    }

    if let Some(schema) = schema {
        if let Some(artifact_type) = extract_artifact_type_from_schema(schema) {
            if let Some(parsed) = parse_artifact_type(&artifact_type) {
                return Ok(parsed);
            }
            return Err(ArtifactError::Transform(format!(
                "Tool '{}' schema has unknown x-artifact-type '{}'. Valid types: text, table, \
                 chart, form, dashboard, presentation_card, list, copy_paste_text, blog",
                tool_name, artifact_type
            )));
        }

        if is_tabular_schema(schema) {
            return Ok(ArtifactType::Table);
        }
        if is_form_schema(schema) {
            return Ok(ArtifactType::Form);
        }
        if is_chart_schema(schema) {
            return Ok(ArtifactType::Chart);
        }
    }

    if is_tabular_data(tool_result) {
        return Ok(ArtifactType::Table);
    }

    Err(ArtifactError::Transform(format!(
        "Tool '{}' missing required x-artifact-type. Add x-artifact-type to tool output or \
         schema. Valid types: text, table, chart, form, dashboard, presentation_card, list, \
         copy_paste_text, blog",
        tool_name
    )))
}

pub fn infer_type_from_result(
    tool_result: &CallToolResult,
    schema: Option<&JsonValue>,
    tool_name: &str,
) -> Result<ArtifactType, ArtifactError> {
    if let Some(structured) = &tool_result.structured_content {
        if let Some(artifact_type) = extract_artifact_type_from_data(structured) {
            if let Some(parsed) = parse_artifact_type(&artifact_type) {
                return Ok(parsed);
            }
            return Err(ArtifactError::Transform(format!(
                "Tool '{}' has unknown x-artifact-type '{}'. Valid types: text, table, chart, \
                 form, dashboard, presentation_card, list, copy_paste_text, blog",
                tool_name, artifact_type
            )));
        }
    }

    if let Some(schema) = schema {
        if let Some(artifact_type) = extract_artifact_type_from_schema(schema) {
            if let Some(parsed) = parse_artifact_type(&artifact_type) {
                return Ok(parsed);
            }
            return Err(ArtifactError::Transform(format!(
                "Tool '{}' schema has unknown x-artifact-type '{}'. Valid types: text, table, \
                 chart, form, dashboard, presentation_card, list, copy_paste_text, blog",
                tool_name, artifact_type
            )));
        }

        if is_tabular_schema(schema) {
            return Ok(ArtifactType::Table);
        }
        if is_form_schema(schema) {
            return Ok(ArtifactType::Form);
        }
        if is_chart_schema(schema) {
            return Ok(ArtifactType::Chart);
        }
    }

    if let Some(structured) = &tool_result.structured_content {
        let (actual_data, _) = unwrap_tool_response(structured);
        if is_tabular_data(actual_data) {
            return Ok(ArtifactType::Table);
        }
    }

    Err(ArtifactError::Transform(format!(
        "Tool '{}' missing required x-artifact-type. Add x-artifact-type to tool output or \
         schema. Valid types: text, table, chart, form, dashboard, presentation_card, list, \
         copy_paste_text, blog",
        tool_name
    )))
}

fn parse_artifact_type(type_str: &str) -> Option<ArtifactType> {
    match type_str.to_lowercase().as_str() {
        "text" => Some(ArtifactType::Text),
        "table" => Some(ArtifactType::Table),
        "chart" => Some(ArtifactType::Chart),
        "form" => Some(ArtifactType::Form),
        "dashboard" => Some(ArtifactType::Dashboard),
        "presentation_card" => Some(ArtifactType::PresentationCard),
        "list" => Some(ArtifactType::List),
        "copy_paste_text" => Some(ArtifactType::CopyPasteText),
        "blog" => Some(ArtifactType::Blog),
        _ => None,
    }
}

fn extract_artifact_type_from_data(data: &JsonValue) -> Option<String> {
    if let Some(t) = data.get("x-artifact-type").and_then(|v| v.as_str()) {
        return Some(t.to_string());
    }

    if let Some(artifact) = data.get("artifact") {
        if let Some(t) = artifact.get("x-artifact-type").and_then(|v| v.as_str()) {
            return Some(t.to_string());
        }
        if let Some(card) = artifact.get("card") {
            if let Some(t) = card.get("x-artifact-type").and_then(|v| v.as_str()) {
                return Some(t.to_string());
            }
        }
    }

    None
}

fn extract_artifact_type_from_schema(schema: &JsonValue) -> Option<String> {
    if let Some(t) = schema.get("x-artifact-type").and_then(|v| v.as_str()) {
        return Some(t.to_string());
    }

    schema
        .get("properties")
        .and_then(|props| props.get("artifact"))
        .and_then(|artifact| artifact.get("x-artifact-type"))
        .and_then(|t| t.as_str())
        .map(String::from)
}

fn is_tabular_schema(schema: &JsonValue) -> bool {
    schema.get("type") == Some(&json!("array"))
        && schema.get("items").and_then(|i| i.get("type")) == Some(&json!("object"))
}

fn is_form_schema(schema: &JsonValue) -> bool {
    if let Some(props) = schema.get("properties") {
        if let Some(fields) = props.get("fields") {
            return fields.get("type") == Some(&json!("array"));
        }
    }
    false
}

fn is_chart_schema(schema: &JsonValue) -> bool {
    if let Some(props) = schema.get("properties") {
        let has_labels = props.get("labels").is_some();
        let has_datasets = props.get("datasets").is_some();
        return has_labels && has_datasets;
    }
    false
}

fn is_tabular_data(data: &JsonValue) -> bool {
    if let Some(arr) = data.as_array() {
        if let Some(first) = arr.first() {
            return first.is_object();
        }
    }
    false
}
