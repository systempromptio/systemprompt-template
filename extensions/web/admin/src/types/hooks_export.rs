use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct HooksFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub hooks: HashMap<HookEventType, Vec<MatcherGroup>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEventType {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    Stop,
    SubagentStart,
    SubagentStop,
    TaskCompleted,
    TeammateIdle,
    Notification,
    ConfigChange,
    WorktreeCreate,
    WorktreeRemove,
    PreCompact,
    InstructionsLoaded,
}

impl fmt::Display for HookEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionStart => write!(f, "SessionStart"),
            Self::SessionEnd => write!(f, "SessionEnd"),
            Self::UserPromptSubmit => write!(f, "UserPromptSubmit"),
            Self::PreToolUse => write!(f, "PreToolUse"),
            Self::PostToolUse => write!(f, "PostToolUse"),
            Self::PostToolUseFailure => write!(f, "PostToolUseFailure"),
            Self::PermissionRequest => write!(f, "PermissionRequest"),
            Self::Stop => write!(f, "Stop"),
            Self::SubagentStart => write!(f, "SubagentStart"),
            Self::SubagentStop => write!(f, "SubagentStop"),
            Self::TaskCompleted => write!(f, "TaskCompleted"),
            Self::TeammateIdle => write!(f, "TeammateIdle"),
            Self::Notification => write!(f, "Notification"),
            Self::ConfigChange => write!(f, "ConfigChange"),
            Self::WorktreeCreate => write!(f, "WorktreeCreate"),
            Self::WorktreeRemove => write!(f, "WorktreeRemove"),
            Self::PreCompact => write!(f, "PreCompact"),
            Self::InstructionsLoaded => write!(f, "InstructionsLoaded"),
        }
    }
}

impl HookEventType {
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "SessionStart" => Some(Self::SessionStart),
            "SessionEnd" => Some(Self::SessionEnd),
            "UserPromptSubmit" => Some(Self::UserPromptSubmit),
            "PreToolUse" => Some(Self::PreToolUse),
            "PostToolUse" => Some(Self::PostToolUse),
            "PostToolUseFailure" => Some(Self::PostToolUseFailure),
            "PermissionRequest" => Some(Self::PermissionRequest),
            "Stop" => Some(Self::Stop),
            "SubagentStart" => Some(Self::SubagentStart),
            "SubagentStop" => Some(Self::SubagentStop),
            "TaskCompleted" => Some(Self::TaskCompleted),
            "TeammateIdle" => Some(Self::TeammateIdle),
            "Notification" => Some(Self::Notification),
            "ConfigChange" => Some(Self::ConfigChange),
            "WorktreeCreate" => Some(Self::WorktreeCreate),
            "WorktreeRemove" => Some(Self::WorktreeRemove),
            "PreCompact" => Some(Self::PreCompact),
            "InstructionsLoaded" => Some(Self::InstructionsLoaded),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SessionStart => "SessionStart",
            Self::SessionEnd => "SessionEnd",
            Self::UserPromptSubmit => "UserPromptSubmit",
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::PostToolUseFailure => "PostToolUseFailure",
            Self::PermissionRequest => "PermissionRequest",
            Self::Stop => "Stop",
            Self::SubagentStart => "SubagentStart",
            Self::SubagentStop => "SubagentStop",
            Self::TaskCompleted => "TaskCompleted",
            Self::TeammateIdle => "TeammateIdle",
            Self::Notification => "Notification",
            Self::ConfigChange => "ConfigChange",
            Self::WorktreeCreate => "WorktreeCreate",
            Self::WorktreeRemove => "WorktreeRemove",
            Self::PreCompact => "PreCompact",
            Self::InstructionsLoaded => "InstructionsLoaded",
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CommandHook {
    pub command: String,
    #[serde(rename = "async", skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}
