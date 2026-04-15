use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl FromStr for ActivityCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "login" => Ok(Self::Login),
            "session" => Ok(Self::Session),
            "prompt" => Ok(Self::Prompt),
            "skill_usage" => Ok(Self::SkillUsage),
            "marketplace_edit" => Ok(Self::MarketplaceEdit),
            "marketplace_connect" => Ok(Self::MarketplaceConnect),
            "user_management" => Ok(Self::UserManagement),
            "tool_usage" => Ok(Self::ToolUsage),
            "error" => Ok(Self::Error),
            "agent_response" => Ok(Self::AgentResponse),
            "notification" => Ok(Self::Notification),
            "task_completion" => Ok(Self::TaskCompletion),
            "compaction" => Ok(Self::Compaction),
            "mcp_access" => Ok(Self::McpAccess),
            other => Err(format!("unknown activity category: {other}")),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ActivityCategory {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ActivityCategory {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Self::from_str(&s).map_err(Into::into)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ActivityCategory {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_ref(), buf)
    }
}

impl FromStr for ActivityAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "logged_in" => Ok(Self::LoggedIn),
            "started" => Ok(Self::Started),
            "ended" => Ok(Self::Ended),
            "submitted" => Ok(Self::Submitted),
            "used" => Ok(Self::Used),
            "created" => Ok(Self::Created),
            "updated" => Ok(Self::Updated),
            "deleted" => Ok(Self::Deleted),
            "imported" => Ok(Self::Imported),
            "uploaded" => Ok(Self::Uploaded),
            "restored" => Ok(Self::Restored),
            "authenticated" => Ok(Self::Authenticated),
            "rejected" => Ok(Self::Rejected),
            other => Err(format!("unknown activity action: {other}")),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ActivityAction {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ActivityAction {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Self::from_str(&s).map_err(Into::into)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ActivityAction {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_ref(), buf)
    }
}

#[must_use]
pub const fn entity_label(entity: ActivityEntity) -> &'static str {
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
