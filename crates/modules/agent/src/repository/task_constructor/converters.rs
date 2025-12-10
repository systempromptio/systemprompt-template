use crate::error::TaskError;
use crate::models::a2a::{DataPart, FilePart, FileWithBytes, Part, TaskState, TextPart};
use crate::models::{MessagePart, TaskRow};
use systemprompt_models::a2a::TaskMetadata;
use systemprompt_traits::RepositoryError;

pub fn construct_metadata(row: &TaskRow) -> Result<Option<TaskMetadata>, RepositoryError> {
    let metadata_json = row
        .metadata
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let agent_name = row.agent_name.clone().unwrap_or_default();

    let mut metadata = serde_json::from_str::<TaskMetadata>(&metadata_json)
        .unwrap_or_else(|_| TaskMetadata::new_agent_message(agent_name.clone()));

    metadata.agent_name = agent_name;
    metadata.created_at = row.created_at.to_rfc3339();
    metadata.updated_at = Some(row.updated_at.to_rfc3339());
    metadata.started_at = row.started_at.map(|dt| dt.to_rfc3339());
    metadata.completed_at = row.completed_at.map(|dt| dt.to_rfc3339());
    metadata.execution_time_ms = row.execution_time_ms.map(|v| v as i64);

    Ok(Some(metadata))
}

pub fn parse_task_state(state_str: &str) -> Result<TaskState, TaskError> {
    match state_str {
        "submitted" => Ok(TaskState::Submitted),
        "working" => Ok(TaskState::Working),
        "input-required" => Ok(TaskState::InputRequired),
        "completed" => Ok(TaskState::Completed),
        "canceled" | "cancelled" => Ok(TaskState::Canceled),
        "failed" => Ok(TaskState::Failed),
        "rejected" => Ok(TaskState::Rejected),
        "auth-required" => Ok(TaskState::AuthRequired),
        "unknown" => Ok(TaskState::Unknown),
        _ => Err(TaskError::InvalidTaskState {
            state: state_str.to_string(),
        }),
    }
}

pub fn build_part_from_row(part_row: &MessagePart) -> Option<Part> {
    match part_row.part_kind.as_str() {
        "text" => {
            let text = part_row.text_content.clone().unwrap_or_default();
            Some(Part::Text(TextPart { text }))
        },
        "data" => {
            let data_value = part_row.data_content.as_ref()?;
            let data = data_value.as_object()?;
            Some(Part::Data(DataPart { data: data.clone() }))
        },
        "file" => Some(Part::File(FilePart {
            file: FileWithBytes {
                name: part_row.file_name.clone(),
                mime_type: part_row.file_mime_type.clone(),
                bytes: part_row.file_bytes.clone().unwrap_or_default(),
            },
        })),
        _ => None,
    }
}

pub fn build_parts_from_rows(part_rows: &[MessagePart]) -> Result<Vec<Part>, RepositoryError> {
    let mut parts = Vec::new();
    for part_row in part_rows {
        let part = match part_row.part_kind.as_str() {
            "text" => {
                let text = part_row.text_content.clone().unwrap_or_default();
                Part::Text(TextPart { text })
            },
            "data" => {
                let data_value = part_row.data_content.as_ref().ok_or_else(|| {
                    RepositoryError::InvalidData("Missing data_content for data part".to_string())
                })?;

                let data = data_value
                    .as_object()
                    .ok_or_else(|| {
                        RepositoryError::InvalidData(
                            "data_content must be a JSON object".to_string(),
                        )
                    })?
                    .clone();

                Part::Data(DataPart { data })
            },
            "file" => Part::File(FilePart {
                file: FileWithBytes {
                    name: part_row.file_name.clone(),
                    mime_type: part_row.file_mime_type.clone(),
                    bytes: part_row.file_bytes.clone().unwrap_or_default(),
                },
            }),
            _ => continue,
        };
        parts.push(part);
    }
    Ok(parts)
}
