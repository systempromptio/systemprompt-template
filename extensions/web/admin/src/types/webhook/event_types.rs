use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookCommonFields {
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub permission_mode: String,
    #[serde(default)]
    pub transcript_path: String,
    #[serde(default)]
    pub hook_event_name: String,
    pub agent_id: Option<String>,
    pub agent_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionStartData {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionEndData {
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPromptSubmitData {
    #[serde(default)]
    pub prompt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PreToolUseData {
    #[serde(default)]
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: serde_json::Value,
    #[serde(default)]
    pub tool_use_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostToolUseData {
    #[serde(default)]
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: serde_json::Value,
    #[serde(default)]
    pub tool_response: serde_json::Value,
    #[serde(default)]
    pub tool_use_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostToolUseFailureData {
    #[serde(default)]
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: serde_json::Value,
    #[serde(default)]
    pub tool_use_id: String,
    #[serde(default)]
    pub error: String,
    pub is_interrupt: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PermissionRequestData {
    #[serde(default)]
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: serde_json::Value,
    pub permission_suggestions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StopData {
    #[serde(default)]
    pub stop_hook_active: bool,
    pub last_assistant_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub struct SubagentStartData {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubagentStopData {
    #[serde(default)]
    pub stop_hook_active: bool,
    #[serde(default)]
    pub agent_transcript_path: String,
    pub last_assistant_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskCompletedData {
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub task_subject: String,
    pub task_description: Option<String>,
    pub teammate_name: Option<String>,
    pub team_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TeammateIdleData {
    #[serde(default)]
    pub teammate_name: String,
    #[serde(default)]
    pub team_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationData {
    #[serde(default)]
    pub message: String,
    pub title: Option<String>,
    #[serde(default)]
    pub notification_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigChangeData {
    #[serde(default)]
    pub source: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorktreeCreateData {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorktreeRemoveData {
    #[serde(default)]
    pub worktree_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PreCompactData {
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub custom_instructions: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstructionsLoadedData {
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub memory_type: String,
    #[serde(default)]
    pub load_reason: String,
    pub globs: Option<serde_json::Value>,
    pub trigger_file_path: Option<String>,
    pub parent_file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum HookEvent {
    SessionStart(SessionStartData),
    SessionEnd(SessionEndData),
    UserPromptSubmit(UserPromptSubmitData),
    PreToolUse(PreToolUseData),
    PostToolUse(PostToolUseData),
    PostToolUseFailure(PostToolUseFailureData),
    PermissionRequest(PermissionRequestData),
    Stop(StopData),
    SubagentStart(SubagentStartData),
    SubagentStop(SubagentStopData),
    TaskCompleted(TaskCompletedData),
    TeammateIdle(TeammateIdleData),
    Notification(NotificationData),
    ConfigChange(ConfigChangeData),
    WorktreeCreate(WorktreeCreateData),
    WorktreeRemove(WorktreeRemoveData),
    PreCompact(PreCompactData),
    InstructionsLoaded(InstructionsLoadedData),
    Unknown(String),
}
