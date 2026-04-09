use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HooksFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub hooks: HashMap<HookEventType, Vec<MatcherGroup>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum HookEventType {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    TaskCompleted,
    SessionStart,
    SessionEnd,
    SubagentStart,
    TeammateIdle,
    Notification,
    ConfigChange,
    WorktreeCreate,
    WorktreeRemove,
    PreCompact,
}

#[derive(Debug, Serialize)]
pub struct MatcherGroup {
    pub matcher: String,
    pub hooks: Vec<HookHandler>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HookHandler {
    Http(HttpHook),
    Command(CommandHook),
}

#[derive(Debug, Serialize)]
pub struct HttpHook {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "allowedEnvVars", skip_serializing_if = "Option::is_none")]
    pub allowed_env_vars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(rename = "async", skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CommandHook {
    pub command: String,
    #[serde(rename = "async", skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}
