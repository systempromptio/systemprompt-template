use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_models::execution::{
    ArtifactCreatedPayload, BroadcastEventData, EventArtifact, EventMessage, EventMessagePart,
    EventTask, EventTaskStatus, ExecutionStepPayload, TaskCompletedPayload, TaskCreatedPayload,
};
use systemprompt_models::ExecutionStep;

use super::WebhookRequest;
use crate::models::a2a::{Artifact, Message, Part, Task};
use crate::repository::{
    ArtifactRepository, ContextRepository, ExecutionStepRepository, TaskRepository,
};

pub async fn load_event_data(
    app_context: &AppContext,
    request: &WebhookRequest,
    _logger: &LogService,
) -> Result<serde_json::Value, anyhow::Error> {
    let db = app_context.db_pool();

    match request.event_type.as_str() {
        "task_completed" => {
            let task_repo = TaskRepository::new(db.clone());
            let artifact_repo = ArtifactRepository::new(db.clone());
            let step_repo = ExecutionStepRepository::new(db.clone());

            use crate::models::a2a::TaskState;
            let timestamp = chrono::Utc::now();
            task_repo
                .update_task_state(&request.entity_id, TaskState::Completed, &timestamp)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to complete task: {}", e))?;

            let task = task_repo
                .get_task(&request.entity_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load task: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("Task not found: {}", request.entity_id))?;

            let artifacts = artifact_repo
                .get_artifacts_by_task(&request.entity_id)
                .await
                .unwrap_or_default();

            let messages = task_repo
                .get_messages_by_task(&request.entity_id)
                .await
                .unwrap_or_default();

            let execution_steps = step_repo
                .list_by_task(&request.entity_id)
                .await
                .unwrap_or_default();

            let payload = TaskCompletedPayload {
                task: task_to_event_task(&task, &messages),
                artifacts: if artifacts.is_empty() {
                    None
                } else {
                    Some(artifacts.iter().map(artifact_to_event_artifact).collect())
                },
                execution_steps: if execution_steps.is_empty() {
                    None
                } else {
                    Some(
                        execution_steps
                            .iter()
                            .map(execution_step_to_summary)
                            .collect(),
                    )
                },
            };

            let data = BroadcastEventData::TaskCompleted(payload);
            let sanitized_data = data.to_data_value();

            validate_json_serializable(&sanitized_data)
                .map_err(|e| anyhow::anyhow!("JSON validation failed: {}", e))?;

            Ok(sanitized_data)
        },
        "artifact_created" => {
            let artifact_repo = ArtifactRepository::new(db.clone());

            let artifact = artifact_repo
                .get_artifact_by_id(&request.entity_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load artifact: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("Artifact not found: {}", request.entity_id))?;

            let payload = ArtifactCreatedPayload {
                artifact: artifact_to_event_artifact(&artifact),
                task_id: Some(artifact.metadata.task_id.to_string()),
            };

            let data = BroadcastEventData::ArtifactCreated(payload);
            Ok(data.to_data_value())
        },
        "message_received" => {
            let pool = db
                .pool_arc()
                .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
            let message = sqlx::query!(
                r#"SELECT m.id, m.message_id, STRING_AGG(mp.id::text, ',') as part_ids
                FROM task_messages m
                LEFT JOIN message_parts mp ON m.message_id = mp.message_id
                WHERE m.message_id = $1
                GROUP BY m.id, m.message_id"#,
                request.entity_id
            )
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load message: {}", e))?;

            if message.is_some() {
                let data = BroadcastEventData::MessageReceived {
                    message_id: request.entity_id.clone(),
                };
                Ok(data.to_data_value())
            } else {
                Err(anyhow::anyhow!("Message not found: {}", request.entity_id))
            }
        },
        "context_updated" => {
            let context_repo = ContextRepository::new(db.clone());

            let context = context_repo
                .get_context(&request.context_id, &request.user_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load context: {}", e))?;

            Ok(json!({
                "context": context,
            }))
        },
        "execution_step" => {
            let step_data = request
                .step_data
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("step_data required for execution_step events"))?;

            let step: ExecutionStep = serde_json::from_value(step_data.clone())
                .map_err(|e| anyhow::anyhow!("Invalid step_data format: {}", e))?;

            let payload = ExecutionStepPayload {
                task_id: step.task_id.clone(),
                step,
            };

            let data = BroadcastEventData::ExecutionStep(payload);
            Ok(data.to_data_value())
        },
        "task_created" => {
            let task_data = request.task_data.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "task_created event MUST include task_data with TaskCreatedPayload. Task ID: \
                     {}",
                    request.entity_id
                )
            })?;

            let payload: TaskCreatedPayload =
                serde_json::from_value(task_data.clone()).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to deserialize TaskCreatedPayload for task {}: {}. Raw data: {:?}",
                        request.entity_id,
                        e,
                        task_data
                    )
                })?;

            if payload.task.history.is_none()
                || payload.task.history.as_ref().map_or(true, |h| h.is_empty())
            {
                return Err(anyhow::anyhow!(
                    "task_created payload has empty history - user message is missing! Task ID: {}",
                    request.entity_id
                ));
            }

            let data = BroadcastEventData::TaskCreated(payload);
            Ok(data.to_data_value())
        },
        _ => Err(anyhow::anyhow!(
            "Unknown event type: {}",
            request.event_type
        )),
    }
}

fn validate_json_serializable(value: &serde_json::Value) -> Result<(), String> {
    const MAX_PAYLOAD_SIZE: usize = 1_000_000;
    const MAX_TEXT_FIELD_SIZE: usize = 100_000;

    let sanitized = sanitize_payload(value, MAX_TEXT_FIELD_SIZE);

    let serialized = serde_json::to_string(&sanitized)
        .map_err(|e| format!("Failed to serialize to string: {e}"))?;

    if serialized.len() > MAX_PAYLOAD_SIZE {
        return Err(format!(
            "Payload too large: {} bytes (max: {})",
            serialized.len(),
            MAX_PAYLOAD_SIZE
        ));
    }

    serde_json::from_str::<serde_json::Value>(&serialized)
        .map_err(|e| format!("Re-parsing failed: {e}"))?;

    Ok(())
}

fn sanitize_payload(value: &serde_json::Value, max_text_size: usize) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            if s.len() > max_text_size {
                serde_json::Value::String(format!(
                    "{}... [truncated from {} bytes]",
                    &s[..max_text_size.min(s.len())],
                    s.len()
                ))
            } else {
                serde_json::Value::String(s.clone())
            }
        },
        serde_json::Value::Array(arr) => serde_json::Value::Array(
            arr.iter()
                .map(|v| sanitize_payload(v, max_text_size))
                .collect(),
        ),
        serde_json::Value::Object(obj) => {
            let sanitized: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), sanitize_payload(v, max_text_size)))
                .collect();
            serde_json::Value::Object(sanitized)
        },
        other => other.clone(),
    }
}

fn task_to_event_task(task: &Task, messages: &[Message]) -> EventTask {
    EventTask {
        id: task.id.clone(),
        context_id: task.context_id.clone(),
        status: EventTaskStatus {
            state: format!("{:?}", task.status.state).to_lowercase(),
            message: task
                .status
                .message
                .as_ref()
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            timestamp: task.status.timestamp,
        },
        history: if messages.is_empty() {
            None
        } else {
            Some(messages.iter().map(message_to_event_message).collect())
        },
        artifacts: task
            .artifacts
            .as_ref()
            .map(|arts| arts.iter().map(artifact_to_event_artifact).collect()),
        metadata: task.metadata.clone(),
        kind: "task".to_string(),
    }
}

fn message_to_event_message(msg: &Message) -> EventMessage {
    EventMessage {
        role: msg.role.to_lowercase(),
        parts: msg.parts.iter().map(part_to_event_part).collect(),
        message_id: msg.message_id.clone(),
    }
}

fn part_to_event_part(part: &Part) -> EventMessagePart {
    match part {
        Part::Text(tp) => EventMessagePart::Text {
            text: tp.text.clone(),
        },
        Part::Data(dp) => EventMessagePart::Data {
            data: serde_json::to_value(&dp.data).unwrap_or_default(),
        },
        Part::File(fp) => EventMessagePart::File {
            file: serde_json::to_value(fp).unwrap_or_default(),
        },
    }
}

fn artifact_to_event_artifact(artifact: &Artifact) -> EventArtifact {
    EventArtifact {
        artifact_id: artifact.artifact_id.clone(),
        name: artifact.name.clone(),
        description: artifact.description.clone(),
        parts: artifact.parts.iter().map(part_to_event_part).collect(),
        metadata: Some(serde_json::to_value(&artifact.metadata).unwrap_or_default()),
    }
}

fn execution_step_to_summary(step: &ExecutionStep) -> ExecutionStep {
    ExecutionStep::from(step.clone())
}
