use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityCategory {
    Login,
    Session,
    Prompt,
    SkillUsage,
    MarketplaceEdit,
    MarketplaceConnect,
    UserManagement,
    ToolUsage,
    Error,
    AgentResponse,
    Notification,
    TaskCompletion,
    Compaction,
    McpAccess,
}

impl fmt::Display for ActivityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for ActivityCategory {
    fn as_ref(&self) -> &str {
        match self {
            Self::Login => "login",
            Self::Session => "session",
            Self::Prompt => "prompt",
            Self::SkillUsage => "skill_usage",
            Self::MarketplaceEdit => "marketplace_edit",
            Self::MarketplaceConnect => "marketplace_connect",
            Self::UserManagement => "user_management",
            Self::ToolUsage => "tool_usage",
            Self::Error => "error",
            Self::AgentResponse => "agent_response",
            Self::Notification => "notification",
            Self::TaskCompletion => "task_completion",
            Self::Compaction => "compaction",
            Self::McpAccess => "mcp_access",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityAction {
    LoggedIn,
    Started,
    Ended,
    Submitted,
    Used,
    Created,
    Updated,
    Deleted,
    Imported,
    Uploaded,
    Restored,
    Authenticated,
    Rejected,
}

impl fmt::Display for ActivityAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for ActivityAction {
    fn as_ref(&self) -> &str {
        match self {
            Self::LoggedIn => "logged_in",
            Self::Started => "started",
            Self::Ended => "ended",
            Self::Submitted => "submitted",
            Self::Used => "used",
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
            Self::Imported => "imported",
            Self::Uploaded => "uploaded",
            Self::Restored => "restored",
            Self::Authenticated => "authenticated",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityEntity {
    Session,
    Skill,
    Plugin,
    Hook,
    McpServer,
    Marketplace,
    User,
    Prompt,
    Agent,
    UserSkill,
    UserAgent,
    UserHook,
    Tool,
}

impl fmt::Display for ActivityEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for ActivityEntity {
    fn as_ref(&self) -> &str {
        match self {
            Self::Session => "session",
            Self::Skill => "skill",
            Self::Plugin => "plugin",
            Self::Hook => "hook",
            Self::McpServer => "mcp_server",
            Self::Marketplace => "marketplace",
            Self::User => "user",
            Self::Prompt => "prompt",
            Self::Agent => "agent",
            Self::UserSkill => "user_skill",
            Self::UserAgent => "user_agent",
            Self::UserHook => "user_hook",
            Self::Tool => "tool",
        }
    }
}

#[must_use]
pub fn entity_label(entity: ActivityEntity) -> &'static str {
    match entity {
        ActivityEntity::Plugin => "plugin",
        ActivityEntity::Hook => "hook",
        ActivityEntity::Agent => "agent",
        ActivityEntity::McpServer => "MCP server",
        ActivityEntity::Skill => "skill",
        ActivityEntity::Marketplace => "marketplace",
        ActivityEntity::User => "user",
        ActivityEntity::Prompt => "prompt",
        ActivityEntity::Session => "session",
        ActivityEntity::UserSkill => "user skill",
        ActivityEntity::UserAgent => "user agent",
        ActivityEntity::UserHook => "user hook",
        ActivityEntity::Tool => "tool",
    }
}
