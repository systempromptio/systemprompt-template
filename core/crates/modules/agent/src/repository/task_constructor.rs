use crate::errors::TaskError;
use crate::models::a2a::{
    Artifact, DataPart, FilePart, FileWithBytes, Message, Part, Task, TaskState, TaskStatus,
    TextPart,
};
use crate::repository::artifact_repository::ArtifactRepository;
use crate::utils::parsing::{optional_string, required_string_task};
use systemprompt_core_database::{DatabaseProvider, DbPool, JsonRow};
use systemprompt_models::a2a::{TaskMetadata, TaskType};
use systemprompt_traits::RepositoryError;

#[derive(Debug, Clone)]
pub struct TaskConstructor {
    db_pool: DbPool,
    artifact_repo: ArtifactRepository,
}

impl TaskConstructor {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            artifact_repo: ArtifactRepository::new(db_pool.clone()),
            db_pool,
        }
    }

    pub async fn construct_task_from_row(&self, row: &JsonRow) -> Result<Task, RepositoryError> {
        let task_id = required_string_task(row, "task_id")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

        let history = self.load_task_messages(&task_id).await?;

        let artifacts = self.load_task_artifacts(&task_id).await?;

        let metadata = self.construct_metadata(row)?;

        let task_state = self
            .parse_task_state(row)
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

        let status_message: Option<Message> = {
            let message_json = optional_string(row, "status_message");
            if let Some(json_str) = message_json {
                if json_str != "null" && !json_str.is_empty() {
                    Some(serde_json::from_str(&json_str)?)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let context_id = required_string_task(row, "context_id")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

        Ok(Task {
            id: task_id.clone().into(),
            context_id: context_id.into(),
            kind: "task".to_string(),
            status: TaskStatus {
                state: task_state,
                message: status_message,
                timestamp: row
                    .get("status_timestamp")
                    .and_then(|v| systemprompt_core_database::parse_database_datetime(v)),
            },
            history,
            artifacts,
            metadata,
        })
    }

    fn construct_metadata(&self, row: &JsonRow) -> Result<Option<TaskMetadata>, RepositoryError> {
        let metadata_json = optional_string(row, "metadata").unwrap_or_else(|| "{}".to_string());
        let created_at = required_string_task(row, "created_at").map_err(|e| {
            RepositoryError::InvalidData(format!("Missing created_at timestamp: {}", e))
        })?;
        let agent_name = required_string_task(row, "agent_name")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
        let updated_at = optional_string(row, "updated_at");
        let started_at = optional_string(row, "started_at");
        let completed_at = optional_string(row, "completed_at");

        let execution_time_ms: Option<i64> = row.get("execution_time_ms").and_then(|v| v.as_i64());

        let mut metadata = match serde_json::from_str::<TaskMetadata>(&metadata_json) {
            Ok(metadata) => metadata,
            Err(_) => TaskMetadata {
                task_type: TaskType::AgentMessage,
                agent_name: String::new(),
                created_at: String::new(),
                updated_at: None,
                started_at: None,
                completed_at: None,
                execution_time_ms: None,
                tool_name: None,
                mcp_server_name: None,
                extensions: None,
            },
        };

        // Always populate from database columns (single source of truth)
        metadata.agent_name = agent_name;
        metadata.created_at = created_at;
        metadata.updated_at = updated_at;
        metadata.started_at = started_at;
        metadata.completed_at = completed_at;
        metadata.execution_time_ms = execution_time_ms;

        Ok(Some(metadata))
    }

    async fn load_task_artifacts(
        &self,
        task_id: &str,
    ) -> Result<Option<Vec<Artifact>>, RepositoryError> {
        let artifacts = self
            .artifact_repo
            .get_artifacts_by_task(task_id)
            .await
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

        if artifacts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(artifacts))
        }
    }

    fn parse_task_state(&self, row: &JsonRow) -> Result<TaskState, TaskError> {
        let state_str = required_string_task(row, "status")?;

        match state_str.as_str() {
            "submitted" => Ok(TaskState::Submitted),
            "working" => Ok(TaskState::Working),
            "input-required" => Ok(TaskState::InputRequired),
            "completed" => Ok(TaskState::Completed),
            "canceled" | "cancelled" => Ok(TaskState::Canceled),
            "failed" => Ok(TaskState::Failed),
            "rejected" => Ok(TaskState::Rejected),
            "auth-required" => Ok(TaskState::AuthRequired),
            "unknown" => Ok(TaskState::Unknown),
            _ => Err(TaskError::InvalidTaskState { state: state_str }),
        }
    }

    async fn load_task_messages(
        &self,
        task_id: &str,
    ) -> Result<Option<Vec<Message>>, RepositoryError> {
        let query = systemprompt_core_database::DatabaseQueryEnum::GetTaskMessages
            .get(self.db_pool.as_ref());
        let message_rows = self.db_pool.as_ref().fetch_all(&query, &[&task_id]).await?;

        if message_rows.is_empty() {
            return Ok(None);
        }

        let mut messages = Vec::new();
        for msg_row in message_rows {
            let message_id = required_string_task(&msg_row, "message_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let role = required_string_task(&msg_row, "role")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            let parts = self.load_message_parts(&message_id, task_id).await?;

            let metadata: Option<serde_json::Value> = {
                let metadata_str = optional_string(&msg_row, "metadata");
                if let Some(json_str) = metadata_str {
                    match serde_json::from_str(&json_str) {
                        Ok(v) if v != serde_json::Value::Null => Some(v),
                        _ => None,
                    }
                } else {
                    None
                }
            };

            let task_id = optional_string(&msg_row, "task_id");
            let context_id = required_string_task(&msg_row, "context_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let client_message_id = optional_string(&msg_row, "client_message_id");

            let reference_task_ids = msg_row
                .get("reference_task_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string().into()))
                        .collect::<Vec<_>>()
                });

            let mut final_metadata = metadata.unwrap_or_else(|| serde_json::json!({}));
            if let Some(client_id) = client_message_id {
                if let Some(obj) = final_metadata.as_object_mut() {
                    obj.insert(
                        "clientMessageId".to_string(),
                        serde_json::Value::String(client_id),
                    );
                }
            }

            messages.push(Message {
                role,
                parts,
                message_id,
                task_id: task_id.map(|id| id.into()),
                context_id: context_id.into(),
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

        Ok(Some(messages))
    }

    async fn load_message_parts(
        &self,
        message_id: &str,
        task_id: &str,
    ) -> Result<Vec<Part>, RepositoryError> {
        let query = systemprompt_core_database::DatabaseQueryEnum::GetMessagePartsByTask
            .get(self.db_pool.as_ref());
        let part_rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&message_id, &task_id])
            .await?;

        let mut parts = Vec::new();
        for part_row in part_rows {
            let part_kind = required_string_task(&part_row, "part_kind")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            let part = match part_kind.as_str() {
                "text" => {
                    let text = optional_string(&part_row, "text_content").unwrap_or_default();
                    Part::Text(TextPart { text })
                },
                "data" => {
                    let data_str = optional_string(&part_row, "data_content").ok_or_else(|| {
                        RepositoryError::InvalidData(
                            "Missing data_content for data part".to_string(),
                        )
                    })?;

                    let data_value: serde_json::Value =
                        serde_json::from_str(&data_str).map_err(|e| {
                            RepositoryError::InvalidData(format!(
                                "Invalid JSON in data_content: {}",
                                e
                            ))
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
                "file" => {
                    let file_name = optional_string(&part_row, "file_name");
                    let file_mime_type = optional_string(&part_row, "file_mime_type");
                    let file_bytes = optional_string(&part_row, "file_bytes").unwrap_or_default();

                    Part::File(FilePart {
                        file: FileWithBytes {
                            name: file_name,
                            mime_type: file_mime_type,
                            bytes: file_bytes,
                        },
                    })
                },
                _ => continue,
            };

            parts.push(part);
        }

        Ok(parts)
    }
}
