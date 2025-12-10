use crate::error::ArtifactError;
use serde_json::{json, Value as JsonValue};
use systemprompt_models::artifacts::types::ArtifactType;
use systemprompt_models::ArtifactMetadata;

use super::helpers::artifact_type_to_string;

pub fn build_metadata(
    artifact_type: &ArtifactType,
    schema: Option<&JsonValue>,
    mcp_execution_id: Option<String>,
    context_id: &str,
    task_id: &str,
    tool_name: &str,
) -> Result<ArtifactMetadata, ArtifactError> {
    use systemprompt_models::{ContextId, TaskId};

    let rendering_hints = match artifact_type {
        ArtifactType::Table => extract_table_hints(schema),
        ArtifactType::Form => extract_form_hints(schema),
        ArtifactType::Chart => extract_chart_hints(schema),
        ArtifactType::PresentationCard => extract_presentation_hints(schema),
        ArtifactType::Dashboard => extract_dashboard_hints(schema),
        _ => json!(null),
    };

    let context_id_typed = ContextId::new(context_id);
    let task_id_typed = TaskId::new(task_id);

    let mut metadata = ArtifactMetadata::new_validated(
        artifact_type_to_string(artifact_type),
        context_id_typed,
        task_id_typed,
    )
    .map_err(|e| ArtifactError::MetadataValidation(format!("{e}")))?;

    metadata = metadata.with_tool_name(tool_name.to_string());

    if !rendering_hints.is_null() {
        metadata = metadata.with_rendering_hints(rendering_hints);
    }

    if let Some(schema) = schema {
        metadata = metadata.with_mcp_schema(schema.clone());
    }

    if let Some(execution_id) = mcp_execution_id {
        metadata = metadata.with_mcp_execution_id(execution_id);
    }

    Ok(metadata)
}

fn extract_table_hints(schema: Option<&JsonValue>) -> JsonValue {
    if let Some(schema) = schema {
        if let Some(hints) = schema.get("x-table-hints") {
            return hints.clone();
        }

        if let Some(items) = schema.get("items") {
            if let Some(properties) = items.get("properties") {
                if let Some(props_obj) = properties.as_object() {
                    let columns: Vec<String> = props_obj.keys().cloned().collect();

                    return json!({
                        "columns": columns,
                        "sortable_columns": columns,
                        "filterable": true,
                        "page_size": 25,
                    });
                }
            }
        }
    }
    json!({})
}

fn extract_form_hints(schema: Option<&JsonValue>) -> JsonValue {
    if let Some(schema) = schema {
        if let Some(hints) = schema.get("x-form-hints") {
            return hints.clone();
        }

        if let Some(properties) = schema.get("properties") {
            let fields = schema_properties_to_form_fields(properties);
            return json!({
                "fields": fields,
                "layout": "vertical",
            });
        }
    }
    json!({})
}

fn extract_chart_hints(schema: Option<&JsonValue>) -> JsonValue {
    if let Some(schema) = schema {
        if let Some(hints) = schema.get("x-chart-hints") {
            return hints.clone();
        }
    }
    json!({
        "chart_type": "bar",
    })
}

fn extract_presentation_hints(schema: Option<&JsonValue>) -> JsonValue {
    if let Some(schema) = schema {
        if let Some(hints) = schema.get("x-presentation-hints") {
            return hints.clone();
        }
    }
    json!({
        "theme": "default"
    })
}

fn extract_dashboard_hints(schema: Option<&JsonValue>) -> JsonValue {
    if let Some(schema) = schema {
        if let Some(hints) = schema.get("x-dashboard-hints") {
            return hints.clone();
        }
    }
    json!({
        "layout": "vertical"
    })
}

fn schema_properties_to_form_fields(properties: &JsonValue) -> Vec<JsonValue> {
    let mut fields = Vec::new();

    if let Some(props_obj) = properties.as_object() {
        for (name, prop_schema) in props_obj {
            let field_type = schema_type_to_form_type(prop_schema);
            let mut field = json!({
                "name": name,
                "type": field_type,
                "label": name,
            });

            if let Some(description) = prop_schema.get("description") {
                field["help_text"] = description.clone();
            }

            if let Some(enum_vals) = prop_schema.get("enum") {
                field["options"] = enum_vals.clone();
            }

            if let Some(default) = prop_schema.get("default") {
                field["default"] = default.clone();
            }

            fields.push(field);
        }
    }

    fields
}

fn schema_type_to_form_type(prop_schema: &JsonValue) -> &str {
    if let Some(format) = prop_schema.get("format").and_then(|f| f.as_str()) {
        return match format {
            "email" => "email",
            "date" => "date",
            "date-time" => "datetime",
            "password" => "password",
            _ => "text",
        };
    }

    if prop_schema.get("enum").is_some() {
        return "select";
    }

    match prop_schema.get("type").and_then(|t| t.as_str()) {
        Some("string") => "text",
        Some("integer") | Some("number") => "number",
        Some("boolean") => "checkbox",
        _ => "text",
    }
}
