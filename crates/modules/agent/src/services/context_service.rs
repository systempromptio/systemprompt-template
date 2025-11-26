use anyhow::{anyhow, Result};
use systemprompt_core_ai::{AiMessage, MessageRole};
use systemprompt_core_database::DbPool;

use crate::models::a2a::{Artifact, Message, Part};
use crate::repository::TaskRepository;

pub struct ContextService {
    task_repo: TaskRepository,
}

impl ContextService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            task_repo: TaskRepository::new(db_pool),
        }
    }

    pub async fn load_conversation_history(&self, context_id: &str) -> Result<Vec<AiMessage>> {
        let tasks = self
            .task_repo
            .list_tasks_by_context(context_id)
            .await
            .map_err(|e| anyhow!("Failed to load conversation history: {}", e))?;

        let mut history_messages = Vec::new();

        for task in tasks {
            if let Some(task_history) = task.history {
                for msg in task_history {
                    let text = Self::extract_message_text(&msg).unwrap_or_default();

                    if text.is_empty() {
                        continue;
                    }

                    let role = match msg.role.as_str() {
                        "user" => MessageRole::User,
                        "agent" => MessageRole::Assistant,
                        _ => continue,
                    };

                    history_messages.push(AiMessage {
                        role,
                        content: text,
                    });
                }
            }

            if let Some(artifacts) = task.artifacts {
                for artifact in artifacts {
                    if let Ok(artifact_content) = Self::serialize_artifact_for_context(&artifact) {
                        history_messages.push(AiMessage {
                            role: MessageRole::Assistant,
                            content: artifact_content,
                        });
                    }
                }
            }
        }

        Ok(history_messages)
    }

    fn extract_message_text(message: &Message) -> Result<String> {
        for part in &message.parts {
            if let Part::Text(text_part) = part {
                return Ok(text_part.text.clone());
            }
        }
        Err(anyhow!("No text content found in message"))
    }

    fn serialize_artifact_for_context(artifact: &Artifact) -> Result<String> {
        let mut content = String::new();

        let artifact_name = artifact
            .name
            .as_ref()
            .unwrap_or(&"unnamed".to_string())
            .clone();

        content.push_str(&format!(
            "[Artifact: {} (type: {})]\n",
            artifact_name, artifact.metadata.artifact_type
        ));

        for part in &artifact.parts {
            match part {
                Part::Text(text_part) => {
                    content.push_str(&text_part.text);
                    content.push('\n');
                }
                Part::Data(data_part) => {
                    let json_str = serde_json::to_string_pretty(&data_part.data)
                        .unwrap_or_else(|_| "{}".to_string());
                    content.push_str(&json_str);
                    content.push('\n');
                }
                Part::File(file_part) => {
                    if let Some(name) = &file_part.file.name {
                        content.push_str(&format!("[File: {}]\n", name));
                    }
                }
            }
        }

        Ok(content)
    }
}
