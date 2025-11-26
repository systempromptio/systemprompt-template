use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
}

impl ExecutionMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_execution_id(mut self, id: String) -> Self {
        self.execution_id = Some(id);
        self
    }

    pub const fn with_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = Some(ts);
        self
    }

    pub fn with_user_id(mut self, id: String) -> Self {
        self.user_id = Some(id);
        self
    }

    pub fn with_session_id(mut self, id: String) -> Self {
        self.session_id = Some(id);
        self
    }

    pub fn with_trace_id(mut self, id: String) -> Self {
        self.trace_id = Some(id);
        self
    }

    pub fn with_skill_id(mut self, id: String) -> Self {
        self.skill_id = Some(id);
        self
    }

    pub fn with_skill_name(mut self, name: String) -> Self {
        self.skill_name = Some(name);
        self
    }

    pub fn with_skill(mut self, id: String, name: String) -> Self {
        self.skill_id = Some(id);
        self.skill_name = Some(name);
        self
    }
}
