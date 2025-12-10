use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{AgentId, SessionId, TaskId, TraceId, UserId};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelPreferences {
    pub hints: Vec<ModelHint>,
    pub cost_priority: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelHint {
    ModelId(String),
    Category(String),
    Provider(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SamplingMetadata {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub user_id: Option<UserId>,
    pub session_id: Option<SessionId>,
    pub trace_id: Option<TraceId>,
    pub agent_id: Option<AgentId>,
    pub task_id: Option<TaskId>,
}
