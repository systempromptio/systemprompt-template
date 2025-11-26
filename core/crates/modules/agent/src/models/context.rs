use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub context_id: String,
    pub user_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContextWithStats {
    pub context_id: String,
    pub user_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub task_count: i64,
    pub message_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContextRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContextRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub message_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub sequence_number: i32,
    pub parts: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDetail {
    pub context: UserContext,
    pub messages: Vec<ContextMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextStateEvent {
    ToolExecutionCompleted {
        context_id: String,
        execution_id: String,
        tool_name: String,
        server_name: String,
        output: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        artifact: Option<super::a2a::artifact::Artifact>,
        status: String,
        timestamp: DateTime<Utc>,
    },
    TaskStatusChanged {
        task: super::a2a::task::Task,
        context_id: String,
        timestamp: DateTime<Utc>,
    },
    ArtifactCreated {
        artifact: super::a2a::artifact::Artifact,
        task_id: String,
        context_id: String,
        timestamp: DateTime<Utc>,
    },
    SkillLoaded {
        skill_id: String,
        skill_name: String,
        description: String,
        request_context: systemprompt_models::execution::context::RequestContext,
        tool_name: Option<String>,
        timestamp: DateTime<Utc>,
    },
    ContextCreated {
        context_id: String,
        context: UserContext,
        timestamp: DateTime<Utc>,
    },
    ContextUpdated {
        context_id: String,
        name: String,
        timestamp: DateTime<Utc>,
    },
    ContextDeleted {
        context_id: String,
        timestamp: DateTime<Utc>,
    },
    Heartbeat {
        timestamp: DateTime<Utc>,
    },
    CurrentAgent {
        context_id: String,
        agent_name: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl ContextStateEvent {
    pub fn context_id(&self) -> Option<&str> {
        match self {
            Self::ToolExecutionCompleted { context_id, .. } => Some(context_id),
            Self::TaskStatusChanged { context_id, .. } => Some(context_id),
            Self::ArtifactCreated { context_id, .. } => Some(context_id),
            Self::SkillLoaded {
                request_context, ..
            } => Some(request_context.context_id().as_str()),
            Self::ContextCreated { context_id, .. } => Some(context_id),
            Self::ContextUpdated { context_id, .. } => Some(context_id),
            Self::ContextDeleted { context_id, .. } => Some(context_id),
            Self::Heartbeat { .. } => None,
            Self::CurrentAgent { context_id, .. } => Some(context_id),
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::ToolExecutionCompleted { timestamp, .. } => *timestamp,
            Self::TaskStatusChanged { timestamp, .. } => *timestamp,
            Self::ArtifactCreated { timestamp, .. } => *timestamp,
            Self::SkillLoaded { timestamp, .. } => *timestamp,
            Self::ContextCreated { timestamp, .. } => *timestamp,
            Self::ContextUpdated { timestamp, .. } => *timestamp,
            Self::ContextDeleted { timestamp, .. } => *timestamp,
            Self::Heartbeat { timestamp } => *timestamp,
            Self::CurrentAgent { timestamp, .. } => *timestamp,
        }
    }
}
