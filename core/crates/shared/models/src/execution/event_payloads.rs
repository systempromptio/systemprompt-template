//! Typed event payloads for broadcast events.
//!
//! These types enforce correct structure for SSE broadcast events,
//! preventing protocol violations like missing `task.history`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ExecutionStep;
use crate::{ContextId, TaskId, TaskMetadata};

/// Task status for event payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTaskStatus {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Message part for event payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EventMessagePart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "data")]
    Data { data: Value },
    #[serde(rename = "file")]
    File { file: Value },
}

/// Message for event payloads (A2A Message structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    pub role: String,
    pub parts: Vec<EventMessagePart>,
    #[serde(rename = "messageId")]
    pub message_id: String,
}

/// Artifact for event payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventArtifact {
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub parts: Vec<EventMessagePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Task for event payloads (A2A Task structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTask {
    pub id: TaskId,
    #[serde(rename = "contextId")]
    pub context_id: ContextId,
    pub status: EventTaskStatus,
    /// History MUST be included for task_completed events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<EventMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<EventArtifact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TaskMetadata>,
    #[serde(rename = "kind")]
    pub kind: String,
}

/// Payload for task_completed events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedPayload {
    pub task: EventTask,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<EventArtifact>>,
    #[serde(rename = "executionSteps", skip_serializing_if = "Option::is_none")]
    pub execution_steps: Option<Vec<ExecutionStep>>,
}

/// Payload for task_created events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreatedPayload {
    pub task: EventTask,
}

/// Payload for task_status_changed events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusChangedPayload {
    pub task: EventTask,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<EventArtifact>>,
}

/// Payload for artifact_created events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactCreatedPayload {
    pub artifact: EventArtifact,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

/// Payload for execution_step events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStepPayload {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub step: ExecutionStep,
}

/// Payload for skill_loaded events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLoadedPayload {
    pub skill_id: String,
    pub skill_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_context: Option<Value>,
}

/// Typed event data enum - enforces correct payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum BroadcastEventData {
    #[serde(rename = "task_completed")]
    TaskCompleted(TaskCompletedPayload),

    #[serde(rename = "task_created")]
    TaskCreated(TaskCreatedPayload),

    #[serde(rename = "task_status_changed")]
    TaskStatusChanged(TaskStatusChangedPayload),

    #[serde(rename = "artifact_created")]
    ArtifactCreated(ArtifactCreatedPayload),

    #[serde(rename = "execution_step")]
    ExecutionStep(ExecutionStepPayload),

    #[serde(rename = "skill_loaded")]
    SkillLoaded(SkillLoadedPayload),

    #[serde(rename = "message_received")]
    MessageReceived { message_id: String },
}

impl BroadcastEventData {
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::TaskCompleted(_) => "task_completed",
            Self::TaskCreated(_) => "task_created",
            Self::TaskStatusChanged(_) => "task_status_changed",
            Self::ArtifactCreated(_) => "artifact_created",
            Self::ExecutionStep(_) => "execution_step",
            Self::SkillLoaded(_) => "skill_loaded",
            Self::MessageReceived { .. } => "message_received",
        }
    }

    /// Convert to untagged JSON Value for backward compatibility
    pub fn to_data_value(&self) -> Value {
        match self {
            Self::TaskCompleted(p) => serde_json::to_value(p).unwrap_or_default(),
            Self::TaskCreated(p) => serde_json::to_value(p).unwrap_or_default(),
            Self::TaskStatusChanged(p) => serde_json::to_value(p).unwrap_or_default(),
            Self::ArtifactCreated(p) => serde_json::to_value(p).unwrap_or_default(),
            Self::ExecutionStep(p) => serde_json::to_value(p).unwrap_or_default(),
            Self::SkillLoaded(p) => serde_json::to_value(p).unwrap_or_default(),
            Self::MessageReceived { message_id } => {
                serde_json::json!({ "message_id": message_id })
            },
        }
    }
}
