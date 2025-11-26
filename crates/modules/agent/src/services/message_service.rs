use anyhow::{anyhow, Result};
use serde_json::json;
use uuid::Uuid;

use crate::models::a2a::{Message, Part, TextPart};
use crate::repository::TaskRepository;
use systemprompt_core_database::{DatabaseProvider, DatabaseTransaction, DbPool};
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_traits::Repository;

pub struct MessageService {
    task_repo: TaskRepository,
    logger: LogService,
}

impl std::fmt::Debug for MessageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageService").finish_non_exhaustive()
    }
}

impl MessageService {
    pub fn new(db_pool: DbPool, logger: LogService) -> Self {
        Self {
            task_repo: TaskRepository::new(db_pool),
            logger,
        }
    }

    pub async fn persist_message_in_tx(
        &self,
        tx: &mut dyn DatabaseTransaction,
        message: &Message,
        task_id: &TaskId,
        context_id: &ContextId,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<i32> {
        let sequence_number = self
            .task_repo
            .get_next_sequence_number_in_tx(tx, task_id.as_str())
            .await?;

        self.task_repo
            .persist_message_with_tx(
                tx,
                message,
                task_id.as_str(),
                context_id.as_str(),
                sequence_number,
                user_id,
                session_id,
                trace_id,
            )
            .await
            .map_err(|e| anyhow!("Failed to persist message: {}", e))?;

        self.logger
            .log(
                LogLevel::Info,
                "message_service",
                "Message persisted",
                Some(json!({
                    "message_id": message.message_id,
                    "task_id": task_id.as_str(),
                    "sequence_number": sequence_number,
                })),
            )
            .await
            .ok();

        Ok(sequence_number)
    }

    pub async fn persist_messages(
        &self,
        task_id: &TaskId,
        context_id: &ContextId,
        messages: Vec<Message>,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<Vec<i32>> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = self.task_repo.pool().as_ref().begin_transaction().await?;
        let mut sequence_numbers = Vec::new();

        self.logger
            .log(
                LogLevel::Info,
                "message_service",
                "Persisting multiple messages",
                Some(json!({
                    "task_id": task_id.as_str(),
                    "message_count": messages.len(),
                })),
            )
            .await
            .ok();

        for message in messages {
            let seq = self
                .persist_message_in_tx(
                    &mut *tx, &message, task_id, context_id, user_id, session_id, trace_id,
                )
                .await?;
            sequence_numbers.push(seq);
        }

        tx.commit().await?;

        self.logger
            .log(
                LogLevel::Info,
                "message_service",
                "Messages persisted successfully",
                Some(json!({
                    "task_id": task_id.as_str(),
                    "sequence_numbers": sequence_numbers,
                })),
            )
            .await
            .ok();

        Ok(sequence_numbers)
    }

    pub async fn create_tool_execution_message(
        &self,
        task_id: &TaskId,
        context_id: &ContextId,
        tool_name: &str,
        tool_args: &serde_json::Value,
        request_context: &RequestContext,
    ) -> Result<(String, i32)> {
        let message_id = Uuid::new_v4().to_string();

        let tool_args_display =
            serde_json::to_string_pretty(tool_args).unwrap_or_else(|_| tool_args.to_string());

        let timestamp = chrono::Utc::now().to_rfc3339();

        let message = Message {
            role: "user".to_string(),
            message_id: message_id.clone(),
            task_id: Some(task_id.clone()),
            context_id: context_id.clone(),
            kind: "message".to_string(),
            parts: vec![Part::Text(TextPart {
                text: format!(
                    "Executed MCP tool: {} with arguments:\n{}\n\nExecution ID: {} at {}",
                    tool_name,
                    tool_args_display,
                    task_id.as_str(),
                    timestamp
                ),
            })],
            metadata: Some(json!({
                "source": "mcp_direct_call",
                "tool_name": tool_name,
                "is_synthetic": true,
                "tool_args": tool_args,
                "execution_timestamp": timestamp,
            })),
            extensions: None,
            reference_task_ids: None,
        };

        let mut tx = self.task_repo.pool().as_ref().begin_transaction().await?;

        let sequence_number = self
            .persist_message_in_tx(
                &mut *tx,
                &message,
                task_id,
                context_id,
                Some(request_context.user_id().as_str()),
                request_context.session_id().as_str(),
                request_context.trace_id().as_str(),
            )
            .await?;

        tx.commit().await?;

        self.logger
            .log(
                LogLevel::Info,
                "message_service",
                "Created synthetic tool execution message",
                Some(json!({
                    "message_id": message_id,
                    "task_id": task_id.as_str(),
                    "tool_name": tool_name,
                    "sequence_number": sequence_number,
                })),
            )
            .await
            .ok();

        Ok((message_id, sequence_number))
    }
}
