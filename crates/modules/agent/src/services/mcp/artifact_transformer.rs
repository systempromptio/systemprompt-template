use crate::models::a2a::artifact::Artifact;
use crate::models::a2a::message::{DataPart, FilePart, FileWithBytes, Part, TextPart};
use rmcp::model::{CallToolResult, RawContent};
use serde_json::{json, Value as JsonValue};
use systemprompt_models::{artifacts::types::ArtifactType, ArtifactMetadata};

/// Extract skill ID from structured_content JSON
/// MCP tool only provides skill_id - lookup details from database
pub fn extract_skill_id(structured_content: &JsonValue) -> Option<String> {
    structured_content
        .get("_skill_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[derive(Debug, Copy, Clone)]
pub struct McpToA2aTransformer;

fn artifact_type_to_string(artifact_type: &ArtifactType) -> String {
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

impl McpToA2aTransformer {
    pub fn transform(
        tool_name: &str,
        tool_result: &CallToolResult,
        output_schema: Option<&JsonValue>,
        context_id: &str,
        task_id: &str,
        tool_arguments: Option<&JsonValue>,
    ) -> Artifact {
        let artifact_type = Self::infer_type_from_result(tool_result, output_schema);

        let execution_id = tool_result
            .structured_content
            .as_ref()
            .and_then(|sc| sc.get("mcp_execution_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let fingerprint = Self::calculate_fingerprint(tool_name, tool_arguments);

        // Extract skill ID from MCP tool response
        let skill_id = tool_result
            .structured_content
            .as_ref()
            .and_then(|sc| extract_skill_id(sc));

        let parts = Self::build_parts_from_result(tool_result);
        let mut metadata = Self::build_metadata(
            &artifact_type,
            output_schema,
            execution_id,
            context_id,
            task_id,
            tool_name,
        );

        metadata = metadata.with_fingerprint(fingerprint);

        // Add skill_id to metadata - artifact repository will lookup full details
        if let Some(sid) = skill_id {
            metadata = metadata.with_skill_id(sid);
        }

        Artifact {
            artifact_id: uuid::Uuid::new_v4().to_string(),
            name: Some(tool_name.to_string()),
            description: None,
            parts,
            metadata,
            extensions: vec![json!(
                "https://systemprompt.io/extensions/artifact-rendering/v1"
            )],
        }
    }

    pub fn transform_from_json(
        tool_name: &str,
        tool_result_json: &JsonValue,
        output_schema: Option<&JsonValue>,
        context_id: &str,
        task_id: &str,
        tool_arguments: Option<&JsonValue>,
    ) -> Artifact {
        let artifact_type = Self::infer_type(tool_result_json, output_schema);

        let execution_id = tool_result_json
            .get("mcp_execution_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let fingerprint = Self::calculate_fingerprint(tool_name, tool_arguments);

        let parts = Self::build_parts(tool_result_json);
        let mut metadata = Self::build_metadata(
            &artifact_type,
            output_schema,
            execution_id,
            context_id,
            task_id,
            tool_name,
        );

        metadata = metadata.with_fingerprint(fingerprint);

        Artifact {
            artifact_id: uuid::Uuid::new_v4().to_string(),
            name: Some(tool_name.to_string()),
            description: None,
            parts,
            metadata,
            extensions: vec![json!(
                "https://systemprompt.io/extensions/artifact-rendering/v1"
            )],
        }
    }

    fn infer_type(tool_result: &JsonValue, schema: Option<&JsonValue>) -> ArtifactType {
        if let Some(schema) = schema {
            if let Some(artifact_type) = schema.get("x-artifact-type") {
                if let Some(type_str) = artifact_type.as_str() {
                    match type_str.to_lowercase().as_str() {
                        "text" => return ArtifactType::Text,
                        "table" => return ArtifactType::Table,
                        "chart" => return ArtifactType::Chart,
                        "form" => return ArtifactType::Form,
                        "dashboard" => return ArtifactType::Dashboard,
                        "presentation_card" => return ArtifactType::PresentationCard,
                        "list" => return ArtifactType::List,
                        "copy_paste_text" => return ArtifactType::CopyPasteText,
                        "blog" => return ArtifactType::Blog,
                        _ => {},
                    }
                }
            }

            if Self::is_tabular_schema(schema) {
                return ArtifactType::Table;
            }
            if Self::is_form_schema(schema) {
                return ArtifactType::Form;
            }
            if Self::is_chart_schema(schema) {
                return ArtifactType::Chart;
            }
        }

        if Self::is_tabular_data(tool_result) {
            return ArtifactType::Table;
        }

        ArtifactType::Text
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

    fn infer_type_from_result(
        tool_result: &CallToolResult,
        schema: Option<&JsonValue>,
    ) -> ArtifactType {
        if let Some(schema) = schema {
            if let Some(artifact_type) = schema.get("x-artifact-type") {
                if let Some(type_str) = artifact_type.as_str() {
                    match type_str.to_lowercase().as_str() {
                        "text" => return ArtifactType::Text,
                        "table" => return ArtifactType::Table,
                        "chart" => return ArtifactType::Chart,
                        "form" => return ArtifactType::Form,
                        "dashboard" => return ArtifactType::Dashboard,
                        "presentation_card" => return ArtifactType::PresentationCard,
                        "list" => return ArtifactType::List,
                        "copy_paste_text" => return ArtifactType::CopyPasteText,
                        "blog" => return ArtifactType::Blog,
                        _ => {},
                    }
                }
            }

            if Self::is_tabular_schema(schema) {
                return ArtifactType::Table;
            }
            if Self::is_form_schema(schema) {
                return ArtifactType::Form;
            }
            if Self::is_chart_schema(schema) {
                return ArtifactType::Chart;
            }
        }

        if let Some(structured) = &tool_result.structured_content {
            if Self::is_tabular_data(structured) {
                return ArtifactType::Table;
            }
        }

        ArtifactType::Text
    }

    fn build_parts_from_result(tool_result: &CallToolResult) -> Vec<Part> {
        let mut parts = Vec::new();

        if let Some(structured) = &tool_result.structured_content {
            eprintln!("[TRANSFORMER] Found structured_content");
            if let Some(obj) = structured.as_object() {
                let mut cleaned_data = obj.clone();

                // Remove metadata fields that belong in artifact.metadata, not data
                cleaned_data.remove("_execution_id");
                cleaned_data.remove("mcp_execution_id");
                cleaned_data.remove("x-artifact-type");

                // Log what we're keeping
                eprintln!(
                    "[TRANSFORMER] Creating Part::Data with {} fields: {:?}",
                    cleaned_data.len(),
                    cleaned_data.keys().collect::<Vec<_>>()
                );

                parts.push(Part::Data(DataPart { data: cleaned_data }));
                eprintln!(
                    "[TRANSFORMER] Parts count after structured_content: {}",
                    parts.len()
                );
                return parts;
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
            return parts;
        }

        panic!(
            "ARTIFACT TRANSFORM ERROR: Tool result must have either 'structured_content' or 'content' array.\nReceived CallToolResult with {} content items and structured_content: {}",
            tool_result.content.len(),
            tool_result.structured_content.is_some()
        );
    }

    fn build_parts(tool_result: &JsonValue) -> Vec<Part> {
        let mut parts = Vec::new();

        if let Some(structured) = tool_result.get("structured_content") {
            let obj = structured.as_object().expect(&format!(
                "ARTIFACT TRANSFORM ERROR: 'structured_content' must be an object, got: {}",
                serde_json::to_string_pretty(structured)
                    .unwrap_or_else(|_| "invalid JSON".to_string())
            ));

            let mut cleaned_data = obj.clone();
            cleaned_data.remove("_execution_id");
            parts.push(Part::Data(DataPart { data: cleaned_data }));
            return parts;
        }

        if tool_result.get("_execution_id").is_some() {
            if let Some(obj) = tool_result.as_object() {
                let mut cleaned_data = obj.clone();
                cleaned_data.remove("_execution_id");
                parts.push(Part::Data(DataPart { data: cleaned_data }));
                return parts;
            }
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
                    return parts;
                }
            }
        }

        panic!(
            "ARTIFACT TRANSFORM ERROR: Tool result must have either 'structured_content' or 'content' array.\nReceived: {}",
            serde_json::to_string_pretty(tool_result).unwrap_or_else(|_| "invalid JSON".to_string())
        );
    }

    fn build_metadata(
        artifact_type: &ArtifactType,
        schema: Option<&JsonValue>,
        mcp_execution_id: Option<String>,
        context_id: &str,
        task_id: &str,
        tool_name: &str,
    ) -> ArtifactMetadata {
        use systemprompt_models::{ContextId, TaskId};

        let rendering_hints = match artifact_type {
            ArtifactType::Table => Self::extract_table_hints(schema),
            ArtifactType::Form => Self::extract_form_hints(schema),
            ArtifactType::Chart => Self::extract_chart_hints(schema),
            ArtifactType::PresentationCard => Self::extract_presentation_hints(schema),
            ArtifactType::Dashboard => Self::extract_dashboard_hints(schema),
            _ => json!(null),
        };

        let context_id_typed = ContextId::new(context_id);
        let task_id_typed = TaskId::new(task_id);

        let mut metadata = ArtifactMetadata::new_validated(
            artifact_type_to_string(artifact_type),
            context_id_typed,
            task_id_typed,
        )
        .unwrap_or_else(|e| panic!("VALIDATION ERROR creating ArtifactMetadata: {}", e));

        metadata = metadata.with_tool_name(tool_name.to_string());

        if !rendering_hints.is_null() {
            metadata = metadata.with_rendering_hints(rendering_hints);
        }

        if let Some(schema) = schema {
            metadata = metadata.with_mcp_schema(schema.clone());

            if let Some(render_behavior) = schema.get("x-render-behavior") {
                if let Some(behavior_str) = render_behavior.as_str() {
                    metadata = metadata.with_render_behavior(behavior_str.to_string());
                }
            }
        }

        if let Some(execution_id) = mcp_execution_id {
            metadata = metadata.with_mcp_execution_id(execution_id);
        }

        metadata
    }

    fn calculate_fingerprint(tool_name: &str, tool_arguments: Option<&JsonValue>) -> String {
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
                let fields = Self::schema_properties_to_form_fields(properties);
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
                let field_type = Self::schema_type_to_form_type(prop_schema);
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
}
