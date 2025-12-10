mod helpers;
mod metadata_builder;
mod parts_builder;
mod type_inference;

use crate::error::ArtifactError;
use crate::models::a2a::artifact::Artifact;
use rmcp::model::CallToolResult;
use serde_json::{json, Value as JsonValue};

pub use helpers::extract_skill_id;

use helpers::{calculate_fingerprint, extract_artifact_id, extract_execution_id};
use metadata_builder::build_metadata;
use parts_builder::{build_parts, build_parts_from_result};
use type_inference::{infer_type, infer_type_from_result};

#[derive(Debug, Copy, Clone)]
pub struct McpToA2aTransformer;

impl McpToA2aTransformer {
    pub fn transform(
        tool_name: &str,
        tool_result: &CallToolResult,
        output_schema: Option<&JsonValue>,
        context_id: &str,
        task_id: &str,
        tool_arguments: Option<&JsonValue>,
    ) -> Result<Artifact, ArtifactError> {
        let artifact_type = infer_type_from_result(tool_result, output_schema, tool_name)?;

        let execution_id = tool_result
            .structured_content
            .as_ref()
            .and_then(extract_execution_id);

        let fingerprint = calculate_fingerprint(tool_name, tool_arguments);

        let skill_id = tool_result
            .structured_content
            .as_ref()
            .and_then(extract_skill_id);

        let parts = build_parts_from_result(tool_result)?;
        let mut metadata = build_metadata(
            &artifact_type,
            output_schema,
            execution_id,
            context_id,
            task_id,
            tool_name,
        )?;

        metadata = metadata.with_fingerprint(fingerprint);

        if let Some(sid) = skill_id {
            metadata = metadata.with_skill_id(sid);
        }

        let artifact_id = tool_result
            .structured_content
            .as_ref()
            .and_then(extract_artifact_id)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(Artifact {
            artifact_id,
            name: Some(tool_name.to_string()),
            description: None,
            parts,
            metadata,
            extensions: vec![json!(
                "https://systemprompt.io/extensions/artifact-rendering/v1"
            )],
        })
    }

    pub fn transform_from_json(
        tool_name: &str,
        tool_result_json: &JsonValue,
        output_schema: Option<&JsonValue>,
        context_id: &str,
        task_id: &str,
        tool_arguments: Option<&JsonValue>,
    ) -> Result<Artifact, ArtifactError> {
        let artifact_type = infer_type(tool_result_json, output_schema, tool_name)?;

        let execution_id = extract_execution_id(tool_result_json);
        let fingerprint = calculate_fingerprint(tool_name, tool_arguments);
        let skill_id = extract_skill_id(tool_result_json);

        let parts = build_parts(tool_result_json)?;
        let mut metadata = build_metadata(
            &artifact_type,
            output_schema,
            execution_id,
            context_id,
            task_id,
            tool_name,
        )?;

        metadata = metadata.with_fingerprint(fingerprint);

        if let Some(sid) = skill_id {
            metadata = metadata.with_skill_id(sid);
        }

        let artifact_id = extract_artifact_id(tool_result_json)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(Artifact {
            artifact_id,
            name: Some(tool_name.to_string()),
            description: None,
            parts,
            metadata,
            extensions: vec![json!(
                "https://systemprompt.io/extensions/artifact-rendering/v1"
            )],
        })
    }
}
