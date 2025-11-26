/*!
 * Task Repository Implementation for A2A Agent Task Management
 *
 * This module provides a comprehensive repository pattern implementation for managing
 * tasks within the Agent-to-Agent (A2A) communication framework following the A2A
 * specification exactly.
 *
 * Key Features:
 * - A2A specification-compliant task serialization and deserialization
 * - JSON-based storage for arrays and complex objects
 * - Atomic task status updates with timestamp tracking
 * - Proper error handling with detailed error types
 *
 * Usage Example:
 * ```rust
 * use systemprompt_core_database::{DbPool, DbPoolExt};
 * let pool = SqlitePool::connect("sqlite::memory:").await?;
 * let repo = TaskRepository::new(pool);
 *
 * // Create a new task
 * let task = Task { ... };
 * let task_id = repo.create_task(&task).await?;
 *
 * // Update task status
 * let status = TaskStatus { state: TaskState::Working, ... };
 * repo.update_task_status(&task_id, &status).await?;
 * ```
 */

use crate::models::a2a::{Message, Part, Task, TaskState};
use crate::repository::task_constructor::TaskConstructor;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};
use systemprompt_core_system::repository::analytics::session::AnalyticsSessionRepository;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug, Clone)]
pub struct TaskRepository {
    db_pool: DbPool,
    analytics_session_repo: AnalyticsSessionRepository,
}

impl TaskRepository {
    /// Create a new task repository
    #[must_use]
    pub fn new(db_pool: DbPool) -> Self {
        let analytics_session_repo = AnalyticsSessionRepository::new(db_pool.clone());
        Self {
            db_pool,
            analytics_session_repo,
        }
    }

    /// Convert `TaskState` enum to database string representation
    const fn task_state_to_db_string(state: TaskState) -> &'static str {
        match state {
            TaskState::Pending => "submitted", // Map legacy "pending" to A2A "submitted"
            TaskState::Submitted => "submitted",
            TaskState::Working => "working",
            TaskState::InputRequired => "input-required",
            TaskState::Completed => "completed",
            TaskState::Canceled => "canceled",
            TaskState::Failed => "failed",
            TaskState::Rejected => "rejected",
            TaskState::AuthRequired => "auth-required",
            TaskState::Unknown => "unknown",
        }
    }

    /// Create a new task in the database following A2A spec exactly
    ///
    /// # Errors
    /// Returns `RepositoryError::SerializationError` if JSON serialization fails
    /// Returns `RepositoryError::DatabaseError` if database insertion fails
    pub async fn create_task(
        &self,
        task: &Task,
        user_id: &systemprompt_identifiers::UserId,
        session_id: &systemprompt_identifiers::SessionId,
        trace_id: &systemprompt_identifiers::TraceId,
        agent_name: &str,
    ) -> Result<String, RepositoryError> {
        let metadata_json = if let Some(ref metadata) = task.metadata {
            serde_json::to_string(metadata)?
        } else {
            "{}".to_string()
        };

        let status = Self::task_state_to_db_string(task.status.state.clone());
        let query = DatabaseQueryEnum::InsertTask.get(self.db_pool.as_ref());

        self.db_pool
            .as_ref()
            .execute(
                &query,
                &[
                    &task.id,
                    &task.context_id,
                    &status,
                    &task.status.timestamp,
                    &user_id.as_ref(),
                    &session_id.as_ref(),
                    &trace_id.as_ref(),
                    &metadata_json,
                    &agent_name,
                ],
            )
            .await?;

        self.analytics_session_repo
            .increment_task_activity(session_id.as_ref(), 1, 0)
            .await
            .ok();

        Ok(task.id.to_string())
    }

    /// Get a task by ID - reconstruct Task from A2A spec-compliant schema
    ///
    /// # Errors
    /// Returns `RepositoryError::SerializationError` if JSON deserialization fails
    /// Returns `RepositoryError::DatabaseError` if database query fails
    pub async fn get_task(&self, task_id: &str) -> Result<Option<Task>, RepositoryError> {
        let query = DatabaseQueryEnum::GetTask.get(self.db_pool.as_ref());
        let task_row = self
            .db_pool
            .as_ref()
            .fetch_optional(&query, &[&task_id])
            .await?;

        let Some(row) = task_row else {
            return Ok(None);
        };

        let constructor = TaskConstructor::new(self.db_pool.clone());
        let task = constructor.construct_task_from_row(&row).await?;

        Ok(Some(task))
    }

    pub async fn list_tasks_by_context(
        &self,
        context_id: &str,
    ) -> Result<Vec<Task>, RepositoryError> {
        let query = DatabaseQueryEnum::ListTasksByContext.get(self.db_pool.as_ref());
        let task_rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&context_id])
            .await?;

        let constructor = TaskConstructor::new(self.db_pool.clone());
        let mut tasks = Vec::new();

        for row in task_rows {
            tasks.push(constructor.construct_task_from_row(&row).await?);
        }

        Ok(tasks)
    }

    pub async fn get_tasks_by_user_id(
        &self,
        user_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Task>, RepositoryError> {
        let task_rows = match (limit, offset) {
            (Some(lim), Some(off)) => {
                let query = DatabaseQueryEnum::GetTasksByUserPaged.get(self.db_pool.as_ref());
                self.db_pool
                    .as_ref()
                    .fetch_all(&query, &[&user_id, &lim, &off])
                    .await?
            },
            (Some(lim), None) => {
                let query = DatabaseQueryEnum::GetTasksByUserPaged.get(self.db_pool.as_ref());
                let offset_zero = 0;
                self.db_pool
                    .as_ref()
                    .fetch_all(&query, &[&user_id, &lim, &offset_zero])
                    .await?
            },
            _ => {
                let query = DatabaseQueryEnum::GetTasksByUser.get(self.db_pool.as_ref());
                self.db_pool.as_ref().fetch_all(&query, &[&user_id]).await?
            },
        };

        let constructor = TaskConstructor::new(self.db_pool.clone());
        let mut tasks = Vec::new();

        for row in &task_rows {
            tasks.push(constructor.construct_task_from_row(row).await?);
        }

        Ok(tasks)
    }

    /// Track agent participation in a context (for multi-agent support)
    ///
    /// # Errors
    /// Returns `RepositoryError::DatabaseError` if database insertion fails
    pub async fn track_agent_in_context(
        &self,
        context_id: &str,
        agent_name: &str,
    ) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::TrackAgentInContext.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(&query, &[&context_id, &agent_name])
            .await?;
        Ok(())
    }

    /// Update task state only (for state transitions during processing)
    ///
    /// When transitioning to `TaskState::Completed`, this method ensures data integrity by:
    /// - Auto-setting `started_at` to CURRENT_TIMESTAMP if NULL (handles edge cases like webhook completions)
    /// - Auto-calculating `execution_time_ms` from started_at to completed_at
    /// - Database constraints enforce that completed tasks always have valid `started_at`
    ///
    /// # Errors
    /// Returns `RepositoryError::DatabaseError` if database update fails
    pub async fn update_task_state(
        &self,
        task_id: &str,
        state: TaskState,
        timestamp: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError> {
        let status = Self::task_state_to_db_string(state);

        let query = if state == TaskState::Completed {
            DatabaseQueryEnum::UpdateTaskStatusCompleted.get(self.db_pool.as_ref())
        } else {
            DatabaseQueryEnum::UpdateTaskStatus.get(self.db_pool.as_ref())
        };

        self.db_pool
            .as_ref()
            .execute(&query, &[&status, &timestamp, &task_id])
            .await?;

        Ok(())
    }

    /// Update task state and save messages (used at task completion)
    ///
    /// This method UPDATES an existing task (must be created first via create_task_simple)
    /// and saves the user and agent messages in a transaction.
    ///
    /// Returns the updated task with all timing metadata populated from the database.
    ///
    /// # Errors
    /// Returns `RepositoryError::SerializationError` if JSON serialization fails
    /// Returns `RepositoryError::DatabaseError` if database update fails
    pub async fn update_task_and_save_messages(
        &self,
        task: &Task,
        user_message: &Message,
        agent_message: &Message,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<Task, RepositoryError> {
        let mut tx = self.db_pool.as_ref().begin_transaction().await?;

        let status = Self::task_state_to_db_string(task.status.state.clone());
        let metadata_json = if let Some(ref metadata) = task.metadata {
            serde_json::to_string(metadata)?
        } else {
            "{}".to_string()
        };

        let query = DatabaseQueryEnum::UpdateTaskWithMetadata.get(self.db_pool.as_ref());
        let rows_affected = tx
            .execute(
                &query,
                &[&status, &task.status.timestamp, &metadata_json, &task.id],
            )
            .await?;

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Task not found for update: {}. This indicates a task_id mismatch between create and update operations.",
                task.id
            )));
        }

        let user_seq = self
            .get_next_sequence_number_in_tx(&mut *tx, task.id.as_str())
            .await?;
        self.persist_message_with_tx(
            &mut *tx,
            user_message,
            task.id.as_str(),
            task.context_id.as_str(),
            user_seq,
            user_id,
            session_id,
            trace_id,
        )
        .await?;

        let agent_seq = self
            .get_next_sequence_number_in_tx(&mut *tx, task.id.as_str())
            .await?;
        self.persist_message_with_tx(
            &mut *tx,
            agent_message,
            task.id.as_str(),
            task.context_id.as_str(),
            agent_seq,
            user_id,
            session_id,
            trace_id,
        )
        .await?;

        tx.commit().await?;

        self.analytics_session_repo
            .increment_task_activity(session_id, 0, 2)
            .await
            .ok();

        let updated_task = self.get_task(task.id.as_str()).await?.ok_or_else(|| {
            RepositoryError::NotFound(format!("Task not found after update: {}", task.id))
        })?;

        Ok(updated_task)
    }

    pub async fn get_next_sequence_number(&self, task_id: &str) -> Result<i32, RepositoryError> {
        let query = DatabaseQueryEnum::GetMaxSequenceNumber.get(self.db_pool.as_ref());
        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(&query, &[&task_id])
            .await?;

        let max_seq = if let Some(ref r) = row {
            r.get("max_seq").and_then(|v| v.as_i64()).map(|v| v as i32)
        } else {
            None
        };

        Ok(max_seq.map(|s| s + 1).unwrap_or(0))
    }

    pub async fn get_next_sequence_number_in_tx(
        &self,
        tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
        task_id: &str,
    ) -> Result<i32, RepositoryError> {
        let query = DatabaseQueryEnum::GetMaxSequenceNumber.get(self.db_pool.as_ref());
        let row = tx.fetch_optional(&query, &[&task_id]).await?;

        let max_seq = if let Some(ref r) = row {
            r.get("max_seq").and_then(|v| v.as_i64()).map(|v| v as i32)
        } else {
            None
        };

        Ok(max_seq.map(|s| s + 1).unwrap_or(0))
    }

    pub async fn get_messages_by_task(
        &self,
        task_id: &str,
    ) -> Result<Vec<Message>, RepositoryError> {
        let query = DatabaseQueryEnum::GetTaskMessages.get(self.db_pool.as_ref());
        let message_rows = self.db_pool.as_ref().fetch_all(&query, &[&task_id]).await?;

        let mut messages = Vec::new();

        for row in message_rows {
            let message_id = row
                .get("message_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::InvalidData("Missing message_id".into()))?
                .to_string();

            let role = row
                .get("role")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::InvalidData("Missing role".into()))?
                .to_string();

            let task_id_opt = row
                .get("task_id")
                .and_then(|v| v.as_str())
                .map(String::from);

            let context_id = row
                .get("context_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::InvalidData("Missing context_id".into()))?;

            let metadata_str = row.get("metadata").and_then(|v| v.as_str()).unwrap_or("{}");

            let metadata: Option<serde_json::Value> = serde_json::from_str(metadata_str)?;

            let reference_task_ids = row
                .get("reference_task_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            item.as_str()
                                .map(|s| systemprompt_identifiers::TaskId::new(s.to_string()))
                        })
                        .collect::<Vec<_>>()
                });

            let parts_query = DatabaseQueryEnum::GetMessageParts.get(self.db_pool.as_ref());
            let part_rows = self
                .db_pool
                .as_ref()
                .fetch_all(&parts_query, &[&message_id])
                .await?;

            let mut parts = Vec::new();

            for part_row in part_rows {
                let part_kind = part_row
                    .get("part_kind")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RepositoryError::InvalidData("Missing part_kind".into()))?;

                let part = match part_kind {
                    "text" => {
                        let text = part_row
                            .get("text_content")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                RepositoryError::InvalidData("Missing text_content".into())
                            })?
                            .to_string();
                        Part::Text(crate::models::a2a::TextPart { text })
                    },
                    "file" => {
                        let name = part_row
                            .get("file_name")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let mime_type = part_row
                            .get("file_mime_type")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let bytes = part_row
                            .get("file_bytes")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                RepositoryError::InvalidData("Missing file_bytes".into())
                            })?
                            .to_string();
                        Part::File(crate::models::a2a::FilePart {
                            file: crate::models::a2a::FileWithBytes {
                                name,
                                mime_type,
                                bytes,
                            },
                        })
                    },
                    "data" => {
                        let data_str = part_row
                            .get("data_content")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                RepositoryError::InvalidData("Missing data_content".into())
                            })?;
                        let data_value: serde_json::Value = serde_json::from_str(data_str)?;
                        let data = if let serde_json::Value::Object(map) = data_value {
                            map
                        } else {
                            return Err(RepositoryError::InvalidData(
                                "Data content must be a JSON object".into(),
                            ));
                        };
                        Part::Data(crate::models::a2a::DataPart { data })
                    },
                    _ => {
                        return Err(RepositoryError::InvalidData(format!(
                            "Unknown part kind: {}",
                            part_kind
                        )));
                    },
                };

                parts.push(part);
            }

            messages.push(Message {
                role,
                message_id,
                task_id: task_id_opt.map(systemprompt_identifiers::TaskId::new),
                context_id: systemprompt_identifiers::ContextId::new(context_id.to_string()),
                kind: "message".to_string(),
                parts,
                metadata,
                extensions: None,
                reference_task_ids,
            });
        }

        Ok(messages)
    }

    pub async fn persist_message_with_tx(
        &self,
        tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
        message: &Message,
        task_id: &str,
        context_id: &str,
        sequence_number: i32,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<(), RepositoryError> {
        let metadata_json = serde_json::to_string(&message.metadata)?;

        let delete_parts_query = DatabaseQueryEnum::DeleteMessageParts.get(self.db_pool.as_ref());
        tx.execute(&delete_parts_query, &[&message.message_id])
            .await?;

        let delete_message_query = DatabaseQueryEnum::DeleteTaskMessage.get(self.db_pool.as_ref());
        tx.execute(&delete_message_query, &[&message.message_id])
            .await?;

        let client_message_id = message
            .metadata
            .as_ref()
            .and_then(|m| m.get("clientMessageId"))
            .and_then(|v| v.as_str());

        let reference_task_ids = message
            .reference_task_ids
            .as_ref()
            .map(|ids| ids.iter().map(|id| id.to_string()).collect::<Vec<String>>());

        let query = DatabaseQueryEnum::InsertMessage.get(self.db_pool.as_ref());
        tx.execute(
            &query,
            &[
                &task_id,
                &message.message_id,
                &client_message_id,
                &message.role,
                &context_id,
                &user_id,
                &session_id,
                &trace_id,
                &sequence_number,
                &metadata_json,
                &reference_task_ids,
            ],
        )
        .await?;

        for (idx, part) in message.parts.iter().enumerate() {
            self.persist_part_with_tx(tx, part, &message.message_id, task_id, idx as i32)
                .await?;
        }

        Ok(())
    }

    async fn persist_part_with_tx(
        &self,
        tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
        part: &Part,
        message_id: &str,
        task_id: &str,
        sequence_number: i32,
    ) -> Result<(), RepositoryError> {
        match part {
            Part::Text(text_part) => {
                let query = DatabaseQueryEnum::InsertMessagePartText.get(self.db_pool.as_ref());
                tx.execute(
                    &query,
                    &[&message_id, &task_id, &sequence_number, &text_part.text],
                )
                .await?;
            },
            Part::File(file_part) => {
                let uri_opt: Option<&str> = None;
                let query = DatabaseQueryEnum::InsertMessagePartFile.get(self.db_pool.as_ref());
                tx.execute(
                    &query,
                    &[
                        &message_id,
                        &task_id,
                        &sequence_number,
                        &file_part.file.name,
                        &file_part.file.mime_type,
                        &uri_opt,
                        &file_part.file.bytes,
                    ],
                )
                .await?;
            },
            Part::Data(data_part) => {
                let data_json = serde_json::to_string(&data_part.data)?;
                let query = DatabaseQueryEnum::InsertMessagePartData.get(self.db_pool.as_ref());
                tx.execute(
                    &query,
                    &[&message_id, &task_id, &sequence_number, &data_json],
                )
                .await?;
            },
        }

        Ok(())
    }

    pub async fn get_messages_by_context(
        &self,
        context_id: &str,
    ) -> Result<Vec<Message>, RepositoryError> {
        let query = DatabaseQueryEnum::GetContextMessages.get(self.db_pool.as_ref());
        let message_rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&context_id])
            .await?;

        let mut messages = Vec::new();

        for row in message_rows {
            let message_id = row
                .get("message_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::InvalidData("Missing message_id".into()))?
                .to_string();

            let role = row
                .get("role")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::InvalidData("Missing role".into()))?
                .to_string();

            let task_id_opt = row
                .get("task_id")
                .and_then(|v| v.as_str())
                .map(String::from);

            let context_id_str = row
                .get("context_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::InvalidData("Missing context_id".into()))?;

            let metadata_str = row.get("metadata").and_then(|v| v.as_str()).unwrap_or("{}");

            let metadata: Option<serde_json::Value> = serde_json::from_str(metadata_str)?;

            let parts_query = DatabaseQueryEnum::GetMessageParts.get(self.db_pool.as_ref());
            let part_rows = self
                .db_pool
                .as_ref()
                .fetch_all(&parts_query, &[&message_id])
                .await?;

            let mut parts = Vec::new();

            for part_row in part_rows {
                let part_kind = part_row
                    .get("part_kind")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RepositoryError::InvalidData("Missing part_kind".into()))?;

                let part = match part_kind {
                    "text" => {
                        let text = part_row
                            .get("text_content")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                RepositoryError::InvalidData("Missing text_content".into())
                            })?
                            .to_string();
                        Part::Text(crate::models::a2a::TextPart { text })
                    },
                    "file" => {
                        let name = part_row
                            .get("file_name")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let mime_type = part_row
                            .get("file_mime_type")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let bytes = part_row
                            .get("file_bytes")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                RepositoryError::InvalidData("Missing file_bytes".into())
                            })?
                            .to_string();
                        Part::File(crate::models::a2a::FilePart {
                            file: crate::models::a2a::FileWithBytes {
                                name,
                                mime_type,
                                bytes,
                            },
                        })
                    },
                    "data" => {
                        let data_str = part_row
                            .get("data_content")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                RepositoryError::InvalidData("Missing data_content".into())
                            })?;
                        let data_value: serde_json::Value = serde_json::from_str(data_str)?;
                        let data = if let serde_json::Value::Object(map) = data_value {
                            map
                        } else {
                            return Err(RepositoryError::InvalidData(
                                "Data content must be a JSON object".into(),
                            ));
                        };
                        Part::Data(crate::models::a2a::DataPart { data })
                    },
                    _ => {
                        return Err(RepositoryError::InvalidData(format!(
                            "Unknown part kind: {}",
                            part_kind
                        )));
                    },
                };

                parts.push(part);
            }

            messages.push(Message {
                role,
                message_id,
                task_id: task_id_opt.map(systemprompt_identifiers::TaskId::new),
                context_id: systemprompt_identifiers::ContextId::new(context_id_str.to_string()),
                kind: "message".to_string(),
                parts,
                metadata,
                extensions: None,
                reference_task_ids: None,
            });
        }

        Ok(messages)
    }
}

impl RepositoryTrait for TaskRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}
