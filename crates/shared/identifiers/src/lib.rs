use serde::{Deserialize, Serialize};
use std::fmt;

/// Session identifier (always required for tracing and analytics)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn system() -> Self {
        Self("system".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Trace identifier (always required for distributed tracing)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TraceId(String);

impl TraceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn system() -> Self {
        Self("system".to_string())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TraceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TraceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for TraceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// User identifier (always has a value, even if "anonymous" or "system")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn anonymous() -> Self {
        Self("anonymous".to_string())
    }

    pub fn system() -> Self {
        Self("system".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_system(&self) -> bool {
        self.0 == "system"
    }

    pub fn is_anonymous(&self) -> bool {
        self.0 == "anonymous"
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for UserId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Context identifier (required for agent operations)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextId(String);

impl ContextId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn system() -> Self {
        Self("system".to_string())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_system(&self) -> bool {
        self.0 == "system"
    }

    pub fn is_anonymous(&self) -> bool {
        self.0 == "anonymous"
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ContextId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ContextId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for ContextId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Task identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(String);

impl TaskId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TaskId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TaskId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for TaskId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for AgentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(String);

impl ClientId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn client_type(&self) -> ClientType {
        if self.0.starts_with("https://") {
            ClientType::Cimd
        } else if self.0.starts_with("sp_") {
            ClientType::FirstParty
        } else if self.0.starts_with("client_") {
            ClientType::ThirdParty
        } else if self.0.starts_with("sys_") {
            ClientType::System
        } else {
            ClientType::Unknown
        }
    }

    pub fn is_dcr(&self) -> bool {
        matches!(
            self.client_type(),
            ClientType::FirstParty | ClientType::ThirdParty
        )
    }

    pub fn is_cimd(&self) -> bool {
        self.0.starts_with("https://")
    }

    pub fn is_system(&self) -> bool {
        self.0.starts_with("sys_")
    }

    pub fn web() -> Self {
        Self("sp_web".to_string())
    }

    pub fn cli() -> Self {
        Self("sp_cli".to_string())
    }

    pub fn mobile_ios() -> Self {
        Self("sp_mobile_ios".to_string())
    }

    pub fn mobile_android() -> Self {
        Self("sp_mobile_android".to_string())
    }

    pub fn desktop() -> Self {
        Self("sp_desktop".to_string())
    }

    pub fn system(service_name: &str) -> Self {
        Self(format!("sys_{service_name}"))
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Agent identifier for request routing and task attribution
///
/// Represents the name/ID of an agent service that handles requests.
/// Unlike [`ClientId`] (OAuth), this identifies which agent service processes
/// the request, not which application made it.
///
/// # Format
/// - Lowercase alphanumeric with hyphens
/// - Examples: "edward", "content-research", "system"
/// - Cannot be empty or "unknown"
///
/// # Examples
/// ```
/// use systemprompt_identifiers::AgentName;
///
/// // Valid agent names
/// let agent = AgentName::new("edward");
/// assert_eq!(agent.as_str(), "edward");
///
/// let agent = AgentName::new("content-research");
/// assert_eq!(agent.as_str(), "content-research");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentName(String);

impl AgentName {
    /// Create a new agent name
    ///
    /// # Panics
    /// - If name is empty
    /// - If name is "unknown" (reserved for error detection)
    ///
    /// # Examples
    /// ```
    /// let agent = AgentName::new("edward");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "Agent name cannot be empty");
        assert_ne!(
            name.to_lowercase().as_str(),
            "unknown",
            "Agent name 'unknown' is reserved for error detection"
        );
        Self(name)
    }

    /// Get the agent name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a system agent name
    pub fn system() -> Self {
        Self("system".to_string())
    }
}

impl AsRef<str> for AgentName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AgentName {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for AgentName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl ToDbValue for AgentName {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &AgentName {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Cimd,
    FirstParty,
    ThirdParty,
    System,
    Unknown,
}

impl ClientType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cimd => "cimd",
            Self::FirstParty => "firstparty",
            Self::ThirdParty => "thirdparty",
            Self::System => "system",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

use systemprompt_core_database::{DbValue, ToDbValue};

impl ToDbValue for SessionId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &SessionId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for TraceId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &TraceId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for UserId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &UserId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for ContextId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &ContextId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for TaskId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &TaskId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for AgentId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &AgentId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for ClientId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &ClientId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

/// AI Provider's tool call identifier (from Anthropic/OpenAI API response)
/// Example: `toolu_01D7XQ2V9K3J8N5M4P6R7T8W9Y`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AiToolCallId(String);

impl AiToolCallId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AiToolCallId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AiToolCallId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AiToolCallId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for AiToolCallId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToDbValue for AiToolCallId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &AiToolCallId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

/// MCP execution identifier (internal tracking in `mcp_tool_executions` table)
/// Example: `550e8400-e29b-41d4-a716-446655440000`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct McpExecutionId(String);

impl McpExecutionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for McpExecutionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for McpExecutionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for McpExecutionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for McpExecutionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToDbValue for McpExecutionId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &McpExecutionId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

/// JWT token (always required after `SessionMiddleware`)
/// Can be user JWT or anonymous JWT
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JwtToken(String);

impl JwtToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JwtToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for JwtToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for JwtToken {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for JwtToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkillId(String);

impl SkillId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SkillId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SkillId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for SkillId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToDbValue for SkillId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &SkillId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SourceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SourceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for SourceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToDbValue for SourceId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &SourceId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CategoryId(String);

impl CategoryId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CategoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CategoryId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CategoryId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for CategoryId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToDbValue for CategoryId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}

impl ToDbValue for &CategoryId {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.0.clone())
    }
}
