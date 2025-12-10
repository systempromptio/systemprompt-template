use crate::models::a2a::{
    Artifact, DataPart, FilePart, FileWithBytes, Message, Part, Task, TaskStatus, TextPart,
};
use crate::models::{
    ArtifactPartRow, ArtifactRow, ExecutionStepBatchRow, MessagePart, TaskMessage, TaskRow,
};
use std::collections::HashMap;
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_models::a2a::ArtifactMetadata;
use systemprompt_models::{ExecutionStep, StepContent, StepId, StepStatus};
use systemprompt_traits::RepositoryError;

use super::{converters, TaskConstructor};

pub async fn construct_tasks_batch(
    constructor: &TaskConstructor,
    task_ids: &[String],
) -> Result<Vec<Task>, RepositoryError> {
    if task_ids.is_empty() {
        return Ok(Vec::new());
    }

    let pool = constructor.get_pg_pool()?;

    let task_rows = super::batch_queries::fetch_task_rows(&pool, task_ids).await?;
    let all_messages = super::batch_queries::fetch_messages(&pool, task_ids).await?;
    let all_parts = super::batch_queries::fetch_message_parts(&pool, task_ids).await?;
    let all_artifact_rows = super::batch_queries::fetch_artifacts(&pool, task_ids).await?;
    let all_execution_steps = super::batch_queries::fetch_execution_steps(&pool, task_ids).await?;

    let artifact_ids: Vec<String> = all_artifact_rows
        .iter()
        .map(|a| a.artifact_id.clone())
        .collect();
    let all_artifact_parts =
        super::batch_queries::fetch_artifact_parts(&pool, &artifact_ids).await?;

    let parts_by_message = group_by_key(&all_parts, |p| p.message_id.clone());
    let messages_by_task = group_by_key(&all_messages, |m| m.task_id.clone());
    let artifacts_by_task = group_by_key(&all_artifact_rows, |a| a.task_id.clone());
    let artifact_parts_by_id = group_by_key(&all_artifact_parts, |p| p.artifact_id.clone());
    let steps_by_task = group_by_key(&all_execution_steps, |s| s.task_id.clone());

    build_tasks(
        &task_rows,
        &messages_by_task,
        &parts_by_message,
        &artifacts_by_task,
        &artifact_parts_by_id,
        &steps_by_task,
    )
}

fn group_by_key<T, F, K>(items: &[T], key_fn: F) -> HashMap<K, Vec<&T>>
where
    F: Fn(&T) -> K,
    K: std::hash::Hash + Eq,
{
    items.iter().fold(HashMap::new(), |mut acc, item| {
        let key = key_fn(item);
        acc.entry(key).or_default().push(item);
        acc
    })
}

fn build_tasks(
    task_rows: &[TaskRow],
    messages_by_task: &HashMap<String, Vec<&TaskMessage>>,
    parts_by_message: &HashMap<String, Vec<&MessagePart>>,
    artifacts_by_task: &HashMap<String, Vec<&ArtifactRow>>,
    artifact_parts_by_id: &HashMap<String, Vec<&ArtifactPartRow>>,
    steps_by_task: &HashMap<String, Vec<&ExecutionStepBatchRow>>,
) -> Result<Vec<Task>, RepositoryError> {
    let mut tasks = Vec::new();

    for row in task_rows {
        let history = build_messages(messages_by_task.get(&row.task_id), parts_by_message);
        let artifacts = build_artifacts(artifacts_by_task.get(&row.task_id), artifact_parts_by_id);
        let execution_steps = build_execution_steps(steps_by_task.get(&row.task_id));

        let mut metadata = converters::construct_metadata(row)?;
        if let Some(ref mut meta) = metadata {
            meta.execution_steps = execution_steps;
        }

        let task_state = converters::parse_task_state(&row.status)
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

        tasks.push(Task {
            id: row.task_id.clone().into(),
            context_id: row.context_id.clone().into(),
            kind: "task".to_string(),
            status: TaskStatus {
                state: task_state,
                message: None,
                timestamp: row.status_timestamp,
            },
            history,
            artifacts,
            metadata,
        });
    }

    Ok(tasks)
}

fn build_execution_steps(
    steps: Option<&Vec<&ExecutionStepBatchRow>>,
) -> Option<Vec<ExecutionStep>> {
    let steps = steps?;
    if steps.is_empty() {
        return None;
    }

    let result: Vec<ExecutionStep> = steps
        .iter()
        .filter_map(|row| {
            let status = row.status.parse::<StepStatus>().ok()?;
            let content: StepContent = serde_json::from_value(row.content.clone()).ok()?;

            Some(ExecutionStep {
                step_id: StepId::from(row.step_id.clone()),
                task_id: row.task_id.clone(),
                status,
                started_at: row.started_at,
                completed_at: row.completed_at,
                duration_ms: row.duration_ms,
                error_message: row.error_message.clone(),
                content,
            })
        })
        .collect();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn build_messages(
    messages: Option<&Vec<&TaskMessage>>,
    parts_by_message: &HashMap<String, Vec<&MessagePart>>,
) -> Option<Vec<Message>> {
    let messages = messages?;
    if messages.is_empty() {
        return None;
    }

    let mut result = Vec::new();
    for msg_row in messages {
        let parts = build_message_parts(parts_by_message.get(&msg_row.message_id));

        let reference_task_ids = msg_row
            .reference_task_ids
            .as_ref()
            .map(|ids| ids.iter().map(|id| id.clone().into()).collect());

        let mut final_metadata = msg_row
            .metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(client_id) = &msg_row.client_message_id {
            if let Some(obj) = final_metadata.as_object_mut() {
                obj.insert(
                    "clientMessageId".to_string(),
                    serde_json::Value::String(client_id.clone()),
                );
            }
        }

        result.push(Message {
            role: msg_row.role.clone(),
            parts,
            message_id: msg_row.message_id.clone(),
            task_id: Some(msg_row.task_id.clone().into()),
            context_id: msg_row.context_id.clone().unwrap_or_default().into(),
            kind: "message".to_string(),
            metadata: if final_metadata == serde_json::json!({}) {
                None
            } else {
                Some(final_metadata)
            },
            extensions: None,
            reference_task_ids,
        });
    }

    Some(result)
}

fn build_message_parts(parts: Option<&Vec<&MessagePart>>) -> Vec<Part> {
    let Some(parts) = parts else {
        return Vec::new();
    };

    parts
        .iter()
        .filter_map(|p| converters::build_part_from_row(p))
        .collect()
}

fn build_artifacts(
    artifacts: Option<&Vec<&ArtifactRow>>,
    artifact_parts_by_id: &HashMap<String, Vec<&ArtifactPartRow>>,
) -> Option<Vec<Artifact>> {
    let artifacts = artifacts?;
    if artifacts.is_empty() {
        return None;
    }

    let mut result = Vec::new();
    for row in artifacts {
        let artifact = build_artifact(row, artifact_parts_by_id);
        result.push(artifact);
    }

    Some(result)
}

fn build_artifact(
    row: &ArtifactRow,
    artifact_parts_by_id: &HashMap<String, Vec<&ArtifactPartRow>>,
) -> Artifact {
    let metadata_value = row
        .metadata
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));

    let metadata = ArtifactMetadata {
        artifact_type: row.artifact_type.clone(),
        context_id: ContextId::new(row.context_id.clone().unwrap_or_default()),
        created_at: row.artifact_created_at.to_rfc3339(),
        task_id: TaskId::new(row.task_id.clone()),
        rendering_hints: metadata_value.get("rendering_hints").cloned(),
        source: row.source.clone(),
        mcp_execution_id: row.mcp_execution_id.clone(),
        mcp_schema: metadata_value.get("mcp_schema").cloned(),
        is_internal: metadata_value.get("is_internal").and_then(|v| v.as_bool()),
        fingerprint: row.fingerprint.clone(),
        tool_name: row.tool_name.clone(),
        execution_index: metadata_value
            .get("execution_index")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize),
        skill_id: row.skill_id.clone(),
        skill_name: row.skill_name.clone(),
    };

    let extensions = metadata_value
        .get("artifact_extensions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_else(|| {
            vec![serde_json::json!(
                "https://systemprompt.io/extensions/artifact-rendering/v1"
            )]
        });

    let parts = build_artifact_parts(artifact_parts_by_id.get(&row.artifact_id));

    Artifact {
        artifact_id: row.artifact_id.clone(),
        name: row.name.clone(),
        description: row.description.clone(),
        parts,
        extensions,
        metadata,
    }
}

fn build_artifact_parts(parts: Option<&Vec<&ArtifactPartRow>>) -> Vec<Part> {
    let Some(parts) = parts else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for row in parts {
        let part = match row.part_kind.as_str() {
            "text" => {
                let text = row.text_content.clone().unwrap_or_default();
                Part::Text(TextPart { text })
            },
            "file" => {
                let bytes = row
                    .file_bytes
                    .clone()
                    .or_else(|| row.file_uri.clone())
                    .unwrap_or_default();
                Part::File(FilePart {
                    file: FileWithBytes {
                        name: row.file_name.clone(),
                        mime_type: row.file_mime_type.clone(),
                        bytes,
                    },
                })
            },
            "data" => {
                let Some(data_value) = &row.data_content else {
                    continue;
                };
                let Some(data) = data_value.as_object() else {
                    continue;
                };
                Part::Data(DataPart { data: data.clone() })
            },
            _ => continue,
        };
        result.push(part);
    }

    result
}
