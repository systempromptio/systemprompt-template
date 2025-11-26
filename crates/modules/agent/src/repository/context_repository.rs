use super::Repository;
use crate::models::context::{ContextStateEvent, UserContext, UserContextWithStats};
use crate::utils::parsing::{
    optional_i64, optional_string, required_datetime_context, required_string_context,
};
use chrono::{DateTime, Utc};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ContextRepository {
    db_pool: DbPool,
    task_repo: std::sync::Arc<super::TaskRepository>,
    artifact_repo: std::sync::Arc<super::ArtifactRepository>,
}

impl ContextRepository {
    #[must_use]
    pub fn new(
        db_pool: DbPool,
        task_repo: std::sync::Arc<super::TaskRepository>,
        artifact_repo: std::sync::Arc<super::ArtifactRepository>,
    ) -> Self {
        Self {
            db_pool,
            task_repo,
            artifact_repo,
        }
    }

    pub async fn create_context(
        &self,
        user_id: &str,
        session_id: &str,
        name: &str,
    ) -> Result<String, RepositoryError> {
        let context_id = Uuid::new_v4().to_string();
        let query = DatabaseQueryEnum::InsertContext.get(self.db_pool.as_ref());

        self.db_pool
            .as_ref()
            .execute(&query, &[&context_id, &user_id, &session_id, &name])
            .await?;

        Ok(context_id)
    }

    pub async fn validate_context_ownership(
        &self,
        context_id: &str,
        user_id: &str,
    ) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::GetContext.get(self.db_pool.as_ref());
        let result = self
            .db_pool
            .as_ref()
            .fetch_optional(&query, &[&context_id, &user_id])
            .await?;

        match result {
            Some(_) => Ok(()),
            None => Err(RepositoryError::NotFound(format!(
                "Context {} not found or user {} does not have access",
                context_id, user_id
            ))),
        }
    }

    pub async fn get_context(
        &self,
        context_id: &str,
        user_id: &str,
    ) -> Result<UserContext, RepositoryError> {
        let query = DatabaseQueryEnum::GetContext.get(self.db_pool.as_ref());
        let row = self
            .db_pool
            .as_ref()
            .fetch_one(&query, &[&context_id, &user_id])
            .await?;

        let context_id = required_string_context(&row, "context_id")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
        let user_id = required_string_context(&row, "user_id")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
        let name = required_string_context(&row, "name")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
        let created_at = required_datetime_context(&row, "created_at")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
        let updated_at = required_datetime_context(&row, "updated_at")
            .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

        Ok(UserContext {
            context_id,
            user_id,
            name,
            created_at,
            updated_at,
        })
    }

    pub async fn list_contexts_basic(
        &self,
        user_id: &str,
    ) -> Result<Vec<UserContext>, RepositoryError> {
        let query = DatabaseQueryEnum::GetContextsByUser.get(self.db_pool.as_ref());
        let rows = self.db_pool.as_ref().fetch_all(&query, &[&user_id]).await?;

        let mut contexts = Vec::new();
        for row in rows {
            let context_id = required_string_context(&row, "context_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let user_id = required_string_context(&row, "user_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let name = required_string_context(&row, "name")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let created_at = required_datetime_context(&row, "created_at")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let updated_at = required_datetime_context(&row, "updated_at")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            contexts.push(UserContext {
                context_id,
                user_id,
                name,
                created_at,
                updated_at,
            });
        }

        Ok(contexts)
    }

    pub async fn list_contexts_with_stats(
        &self,
        user_id: &str,
    ) -> Result<Vec<UserContextWithStats>, RepositoryError> {
        let query = DatabaseQueryEnum::SearchContexts.get(self.db_pool.as_ref());
        let rows = self.db_pool.as_ref().fetch_all(&query, &[&user_id]).await?;

        let mut contexts = Vec::new();
        for row in rows {
            let context_id = required_string_context(&row, "context_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let user_id = required_string_context(&row, "user_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let name = required_string_context(&row, "name")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let created_at = required_datetime_context(&row, "created_at")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let updated_at = required_datetime_context(&row, "updated_at")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let task_count = optional_i64(&row, "task_count").unwrap_or(0);
            let message_count = optional_i64(&row, "message_count").unwrap_or(0);
            let last_message_at = row
                .get("last_message_at")
                .and_then(|v| systemprompt_core_database::parse_database_datetime(v));

            contexts.push(UserContextWithStats {
                context_id,
                user_id,
                name,
                created_at,
                updated_at,
                task_count,
                message_count,
                last_message_at,
            });
        }

        Ok(contexts)
    }

    pub async fn update_context_name(
        &self,
        context_id: &str,
        user_id: &str,
        name: &str,
    ) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::UpdateContext.get(self.db_pool.as_ref());
        let result = self
            .db_pool
            .as_ref()
            .execute(&query, &[&name, &context_id, &user_id])
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Context {} not found for user {}",
                context_id, user_id
            )));
        }

        Ok(())
    }

    pub async fn delete_context(
        &self,
        context_id: &str,
        user_id: &str,
    ) -> Result<(), RepositoryError> {
        let query = DatabaseQueryEnum::DeleteContext.get(self.db_pool.as_ref());
        let result = self
            .db_pool
            .as_ref()
            .execute(&query, &[&context_id, &user_id])
            .await?;

        if result == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Context {} not found for user {}",
                context_id, user_id
            )));
        }

        Ok(())
    }

    pub async fn get_context_events_since(
        &self,
        user_id: &str,
        context_id: &str,
        last_seen: DateTime<Utc>,
    ) -> Result<Vec<ContextStateEvent>, RepositoryError> {
        let mut events = Vec::new();

        let query = DatabaseQueryEnum::GetNewToolExecutionsSince.get(self.db_pool.as_ref());
        let tool_rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&user_id, &context_id, &last_seen])
            .await?;

        for row in tool_rows {
            let output_str = optional_string(&row, "output");
            let _schema_str = optional_string(&row, "output_schema");
            let tool_name = required_string_context(&row, "tool_name")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            let artifact = None;

            let execution_id = required_string_context(&row, "execution_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let server_name = required_string_context(&row, "server_name")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let status = required_string_context(&row, "status")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let timestamp = required_datetime_context(&row, "created_at")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            events.push(ContextStateEvent::ToolExecutionCompleted {
                context_id: context_id.to_string(),
                execution_id,
                tool_name,
                server_name,
                output: output_str,
                artifact,
                status,
                timestamp,
            });
        }

        let query = DatabaseQueryEnum::GetNewTaskStatusChangesSince.get(self.db_pool.as_ref());
        let task_rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&user_id, &context_id, &last_seen])
            .await?;

        for row in task_rows {
            let task_id = required_string_context(&row, "task_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let timestamp = required_datetime_context(&row, "timestamp")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            if let Ok(Some(task)) = self.task_repo.get_task(&task_id).await {
                events.push(ContextStateEvent::TaskStatusChanged {
                    task,
                    context_id: context_id.to_string(),
                    timestamp,
                });
            }
        }

        let query = DatabaseQueryEnum::GetNewArtifactsSince.get(self.db_pool.as_ref());
        let artifact_rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&user_id, &context_id, &last_seen])
            .await?;

        for row in artifact_rows {
            let artifact_id = required_string_context(&row, "artifact_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let task_id = required_string_context(&row, "task_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let timestamp = required_datetime_context(&row, "created_at")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            if let Ok(Some(artifact)) = self.artifact_repo.get_artifact_by_id(&artifact_id).await {
                events.push(ContextStateEvent::ArtifactCreated {
                    artifact,
                    task_id: task_id,
                    context_id: context_id.to_string(),
                    timestamp,
                });
            }
        }

        let query = DatabaseQueryEnum::GetContextUpdatesSince.get(self.db_pool.as_ref());
        let context_update_rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&user_id, &context_id, &last_seen])
            .await?;

        for row in context_update_rows {
            let ctx_id = required_string_context(&row, "context_id")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let name = required_string_context(&row, "name")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;
            let timestamp = required_datetime_context(&row, "updated_at")
                .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

            events.push(ContextStateEvent::ContextUpdated {
                context_id: ctx_id,
                name,
                timestamp,
            });
        }

        events.sort_by(|a, b| a.timestamp().cmp(&b.timestamp()));

        Ok(events)
    }
}

impl Repository for ContextRepository {
    fn pool(&self) -> &DbPool {
        &self.db_pool
    }
}

impl RepositoryTrait for ContextRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}
