use crate::ai::ToolModelConfig;
use crate::auth::{AuthenticatedUser, UserType};
use anyhow::anyhow;
use axum::http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::{Duration, Instant};
use systemprompt_core_logging::LogContext;
use systemprompt_identifiers::{
    AgentName, AiToolCallId, ClientId, ContextId, JwtToken, McpExecutionId, SessionId, TaskId,
    TraceId, UserId,
};
use systemprompt_traits::{ContextPropagation, InjectContextHeaders};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallSource {
    Agentic,
    Direct,
    Ephemeral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub auth_token: JwtToken,
    pub user_id: UserId,
    pub user_type: UserType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub session_id: SessionId,
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub client_id: Option<ClientId>,
    pub is_tracked: bool,
}

impl Default for RequestMetadata {
    fn default() -> Self {
        Self {
            session_id: SessionId::new("unknown".to_string()),
            timestamp: Instant::now(),
            client_id: None,
            is_tracked: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub trace_id: TraceId,
    pub context_id: ContextId,
    pub task_id: Option<TaskId>,
    pub ai_tool_call_id: Option<AiToolCallId>,
    pub mcp_execution_id: Option<McpExecutionId>,
    pub call_source: Option<CallSource>,
    pub agent_name: AgentName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_model_config: Option<ToolModelConfig>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            trace_id: TraceId::new(format!("trace_{}", uuid::Uuid::new_v4())),
            context_id: ContextId::new(String::new()),
            task_id: None,
            ai_tool_call_id: None,
            mcp_execution_id: None,
            call_source: None,
            agent_name: AgentName::system(),
            tool_model_config: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ExecutionSettings {
    pub max_budget_cents: Option<i32>,
    pub user_interaction_mode: Option<UserInteractionMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserInteractionMode {
    Interactive,
    NonInteractive,
}

impl FromStr for CallSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "agentic" => Ok(Self::Agentic),
            "direct" => Ok(Self::Direct),
            "ephemeral" => Ok(Self::Ephemeral),
            _ => Err(anyhow!("Invalid CallSource: {s}")),
        }
    }
}

impl CallSource {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Agentic => "agentic",
            Self::Direct => "direct",
            Self::Ephemeral => "ephemeral",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub auth: AuthContext,
    pub request: RequestMetadata,
    pub execution: ExecutionContext,
    pub settings: ExecutionSettings,

    #[serde(skip)]
    pub user: Option<AuthenticatedUser>,

    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
}

impl RequestContext {
    /// Creates a new `RequestContext` - the ONLY way to construct a context.
    ///
    /// This is the single constructor for `RequestContext`. All contexts must
    /// be created through this method, ensuring consistent initialization.
    ///
    /// # Required Fields
    /// - `session_id`: Identifies the user session
    /// - `trace_id`: For distributed tracing
    /// - `context_id`: Conversation/execution context (empty string for
    ///   user-level contexts)
    /// - `agent_name`: The agent handling this request (use
    ///   `AgentName::system()` for system operations)
    ///
    /// # Optional Fields
    /// Use builder methods to set optional fields:
    /// - `.with_user_id()` - Set the authenticated user
    /// - `.with_auth_token()` - Set the JWT token
    /// - `.with_user_type()` - Set user type (Admin, Standard, Anon)
    /// - `.with_task_id()` - Set task ID for AI operations
    /// - `.with_client_id()` - Set client ID
    /// - `.with_call_source()` - Set call source (Agentic, Direct, Ephemeral)
    ///
    /// # Example
    /// ```
    /// # use systemprompt_models::execution::context::RequestContext;
    /// # use systemprompt_identifiers::{SessionId, TraceId, ContextId, AgentName, UserId};
    /// # use systemprompt_models::auth::UserType;
    /// let ctx = RequestContext::new(
    ///     SessionId::new("sess_123".to_string()),
    ///     TraceId::new("trace_456".to_string()),
    ///     ContextId::new("ctx_789".to_string()),
    ///     AgentName::new("my-agent".to_string()),
    /// )
    /// .with_user_id(UserId::new("user_123".to_string()))
    /// .with_auth_token("jwt_token_here")
    /// .with_user_type(UserType::Standard);
    /// ```
    pub fn new(
        session_id: SessionId,
        trace_id: TraceId,
        context_id: ContextId,
        agent_name: AgentName,
    ) -> Self {
        Self {
            auth: AuthContext {
                auth_token: JwtToken::new(""),
                user_id: UserId::anonymous(),
                user_type: UserType::Anon,
            },
            request: RequestMetadata {
                session_id,
                timestamp: Instant::now(),
                client_id: None,
                is_tracked: true,
            },
            execution: ExecutionContext {
                trace_id,
                context_id,
                task_id: None,
                ai_tool_call_id: None,
                mcp_execution_id: None,
                call_source: None,
                agent_name,
                tool_model_config: None,
            },
            settings: ExecutionSettings::default(),
            user: None,
            start_time: Instant::now(),
        }
    }

    pub fn with_user(mut self, user: AuthenticatedUser) -> Self {
        self.auth.user_id = UserId::new(user.id.to_string());
        self.user = Some(user);
        self
    }

    pub fn with_user_id(mut self, user_id: UserId) -> Self {
        self.auth.user_id = user_id;
        self
    }

    pub fn with_agent_name(mut self, agent_name: AgentName) -> Self {
        self.execution.agent_name = agent_name;
        self
    }

    pub fn with_context_id(mut self, context_id: ContextId) -> Self {
        self.execution.context_id = context_id;
        self
    }

    pub fn with_task_id(mut self, task_id: TaskId) -> Self {
        self.execution.task_id = Some(task_id);
        self
    }

    pub fn with_task(mut self, task_id: TaskId, call_source: CallSource) -> Self {
        self.execution.task_id = Some(task_id);
        self.execution.call_source = Some(call_source);
        self
    }

    pub fn with_ai_tool_call_id(mut self, ai_tool_call_id: AiToolCallId) -> Self {
        self.execution.ai_tool_call_id = Some(ai_tool_call_id);
        self
    }

    pub fn with_mcp_execution_id(mut self, mcp_execution_id: McpExecutionId) -> Self {
        self.execution.mcp_execution_id = Some(mcp_execution_id);
        self
    }

    pub fn with_client_id(mut self, client_id: ClientId) -> Self {
        self.request.client_id = Some(client_id);
        self
    }

    pub const fn with_user_type(mut self, user_type: UserType) -> Self {
        self.auth.user_type = user_type;
        self
    }

    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth.auth_token = JwtToken::new(token.into());
        self
    }

    pub const fn with_call_source(mut self, call_source: CallSource) -> Self {
        self.execution.call_source = Some(call_source);
        self
    }

    pub const fn with_budget(mut self, cents: i32) -> Self {
        self.settings.max_budget_cents = Some(cents);
        self
    }

    pub const fn with_interaction_mode(mut self, mode: UserInteractionMode) -> Self {
        self.settings.user_interaction_mode = Some(mode);
        self
    }

    pub const fn with_tracked(mut self, is_tracked: bool) -> Self {
        self.request.is_tracked = is_tracked;
        self
    }

    pub fn with_tool_model_config(mut self, config: ToolModelConfig) -> Self {
        self.execution.tool_model_config = Some(config);
        self
    }

    pub const fn tool_model_config(&self) -> Option<&ToolModelConfig> {
        self.execution.tool_model_config.as_ref()
    }

    pub const fn session_id(&self) -> &SessionId {
        &self.request.session_id
    }

    pub const fn user_id(&self) -> &UserId {
        &self.auth.user_id
    }

    pub const fn trace_id(&self) -> &TraceId {
        &self.execution.trace_id
    }

    pub const fn context_id(&self) -> &ContextId {
        &self.execution.context_id
    }

    pub const fn agent_name(&self) -> &AgentName {
        &self.execution.agent_name
    }

    pub const fn auth_token(&self) -> &JwtToken {
        &self.auth.auth_token
    }

    pub const fn user_type(&self) -> UserType {
        self.auth.user_type
    }

    pub const fn task_id(&self) -> Option<&TaskId> {
        self.execution.task_id.as_ref()
    }

    pub const fn client_id(&self) -> Option<&ClientId> {
        self.request.client_id.as_ref()
    }

    pub const fn ai_tool_call_id(&self) -> Option<&AiToolCallId> {
        self.execution.ai_tool_call_id.as_ref()
    }

    pub const fn mcp_execution_id(&self) -> Option<&McpExecutionId> {
        self.execution.mcp_execution_id.as_ref()
    }

    pub const fn call_source(&self) -> Option<CallSource> {
        self.execution.call_source
    }

    pub const fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    pub fn is_system(&self) -> bool {
        self.auth.user_id.is_system() && self.execution.context_id.is_system()
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn validate_task_execution(&self) -> Result<(), String> {
        if self.execution.task_id.is_none() {
            return Err("Missing task_id for task execution".to_string());
        }
        if self.execution.context_id.as_str().is_empty() {
            return Err("Missing context_id for task execution".to_string());
        }
        Ok(())
    }

    pub fn validate_authenticated(&self) -> Result<(), String> {
        if self.auth.auth_token.as_str().is_empty() {
            return Err("Missing authentication token".to_string());
        }
        if self.auth.user_id.is_anonymous() {
            return Err("User is not authenticated".to_string());
        }
        Ok(())
    }

    pub fn log_context(&self) -> LogContext {
        let mut log_ctx = LogContext::new()
            .with_session_id(self.request.session_id.as_str())
            .with_trace_id(self.execution.trace_id.as_str())
            .with_user_id(self.auth.user_id.as_str());

        if !self.execution.context_id.as_str().is_empty() {
            log_ctx = log_ctx.with_context_id(self.execution.context_id.as_str());
        }

        if let Some(ref task_id) = self.execution.task_id {
            log_ctx = log_ctx.with_task_id(task_id.as_str());
        }

        if let Some(ref client_id) = self.request.client_id {
            log_ctx = log_ctx.with_client_id(client_id.as_str());
        }

        log_ctx
    }
}

#[derive(Debug, Error)]
pub enum ContextExtractionError {
    #[error("Missing required header: {0}")]
    MissingHeader(String),

    #[error("Missing Authorization header")]
    MissingAuthHeader,

    #[error("Invalid JWT token: {0}")]
    InvalidToken(String),

    #[error("JWT missing required 'session_id' claim")]
    MissingSessionId,

    #[error("JWT missing required 'sub' (user_id) claim")]
    MissingUserId,

    #[error(
        "Missing required 'x-context-id' header (for MCP routes) or contextId in body (for A2A \
         routes)"
    )]
    MissingContextId,

    #[error("Invalid header value: {header}, reason: {reason}")]
    InvalidHeaderValue { header: String, reason: String },

    #[error("Invalid user_id: {0}")]
    InvalidUserId(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Forbidden header '{header}': {reason}")]
    ForbiddenHeader { header: String, reason: String },
}

impl InjectContextHeaders for RequestContext {
    fn inject_headers(&self, headers: &mut HeaderMap) {
        if let Ok(val) = HeaderValue::from_str(self.request.session_id.as_str()) {
            headers.insert("x-session-id", val);
        }
        if let Ok(val) = HeaderValue::from_str(self.execution.trace_id.as_str()) {
            headers.insert("x-trace-id", val);
        }
        if let Ok(val) = HeaderValue::from_str(self.auth.user_id.as_str()) {
            headers.insert("x-user-id", val);
        }
        if let Ok(val) = HeaderValue::from_str(self.auth.user_type.as_str()) {
            headers.insert("x-user-type", val);
        }
        if !self.execution.context_id.as_str().is_empty() {
            if let Ok(val) = HeaderValue::from_str(self.execution.context_id.as_str()) {
                headers.insert("x-context-id", val);
            }
        }
        if let Some(ref task_id) = self.execution.task_id {
            if let Ok(val) = HeaderValue::from_str(task_id.as_str()) {
                headers.insert("x-task-id", val);
            }
        }
        if let Some(ref ai_tool_call_id) = self.execution.ai_tool_call_id {
            if let Ok(val) = HeaderValue::from_str(ai_tool_call_id.as_ref()) {
                headers.insert("x-ai-tool-call-id", val);
            }
        }
        if let Some(ref call_source) = self.execution.call_source {
            if let Ok(val) = HeaderValue::from_str(call_source.as_str()) {
                headers.insert("x-call-source", val);
            }
        }

        if let Ok(val) = HeaderValue::from_str(self.execution.agent_name.as_str()) {
            headers.insert("x-agent-name", val);
        }

        if let Some(ref client_id) = self.request.client_id {
            if let Ok(val) = HeaderValue::from_str(client_id.as_str()) {
                headers.insert("x-client-id", val);
            }
        }

        if !self.auth.auth_token.as_str().is_empty() {
            let auth_value = format!("Bearer {}", self.auth.auth_token.as_str());
            if let Ok(val) = HeaderValue::from_str(&auth_value) {
                headers.insert("authorization", val);
            }
        }
    }
}

impl ContextPropagation for RequestContext {
    fn from_headers(headers: &HeaderMap) -> anyhow::Result<Self> {
        let session_id = headers
            .get("x-session-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow!("Missing x-session-id header"))?;

        let trace_id = headers
            .get("x-trace-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow!("Missing x-trace-id header"))?;

        let user_id = headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow!("Missing x-user-id header"))?;

        let context_id = headers
            .get("x-context-id")
            .and_then(|v| v.to_str().ok())
            .map_or_else(
                || ContextId::new(String::new()),
                |s| ContextId::new(s.to_string()),
            );

        let agent_name = headers
            .get("x-agent-name")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                anyhow!("Missing x-agent-name header - all requests must have agent context")
            })?;

        let task_id = headers
            .get("x-task-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| TaskId::new(s.to_string()));

        let ai_tool_call_id = headers
            .get("x-ai-tool-call-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| AiToolCallId::from(s.to_string()));

        let call_source = headers
            .get("x-call-source")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| CallSource::from_str(s).ok());

        let client_id = headers
            .get("x-client-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| ClientId::new(s.to_string()));

        let mut ctx = Self::new(
            SessionId::new(session_id.to_string()),
            TraceId::new(trace_id.to_string()),
            context_id,
            AgentName::new(agent_name.to_string()),
        )
        .with_user_id(UserId::new(user_id.to_string()));

        if let Some(tid) = task_id {
            ctx = ctx.with_task_id(tid);
        }

        if let Some(ai_id) = ai_tool_call_id {
            ctx = ctx.with_ai_tool_call_id(ai_id);
        }

        if let Some(cs) = call_source {
            ctx = ctx.with_call_source(cs);
        }

        if let Some(cid) = client_id {
            ctx = ctx.with_client_id(cid);
        }

        Ok(ctx)
    }

    fn to_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        self.inject_headers(&mut headers);
        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_basic() {
        let ctx = RequestContext::new(
            SessionId::new("sess_123".to_string()),
            TraceId::new("trace_456".to_string()),
            ContextId::new("ctx_789".to_string()),
            AgentName::new("test-agent".to_string()),
        )
        .with_user_id(UserId::new("user_123".to_string()));

        let headers = ctx.to_headers();
        let ctx2 = RequestContext::from_headers(&headers).unwrap();

        assert_eq!(
            ctx.request.session_id.as_str(),
            ctx2.request.session_id.as_str()
        );
        assert_eq!(
            ctx.execution.trace_id.as_str(),
            ctx2.execution.trace_id.as_str()
        );
        assert_eq!(ctx.auth.user_id.as_str(), ctx2.auth.user_id.as_str());
        assert_eq!(
            ctx.execution.context_id.as_str(),
            ctx2.execution.context_id.as_str()
        );
        assert_eq!(
            ctx.execution.agent_name.as_str(),
            ctx2.execution.agent_name.as_str()
        );
    }

    #[test]
    fn test_round_trip_with_optional_fields() {
        let ctx = RequestContext::new(
            SessionId::new("sess_123".to_string()),
            TraceId::new("trace_456".to_string()),
            ContextId::new("ctx_789".to_string()),
            AgentName::new("test-agent".to_string()),
        )
        .with_user_id(UserId::new("user_123".to_string()))
        .with_task_id(TaskId::new("task_456".to_string()))
        .with_client_id(ClientId::new("client_789".to_string()))
        .with_call_source(CallSource::Direct);

        let headers = ctx.to_headers();
        let ctx2 = RequestContext::from_headers(&headers).unwrap();

        assert_eq!(
            ctx.execution.task_id.as_ref().map(|t| t.as_str()),
            ctx2.execution.task_id.as_ref().map(|t| t.as_str())
        );
        assert_eq!(
            ctx.request.client_id.as_ref().map(|c| c.as_str()),
            ctx2.request.client_id.as_ref().map(|c| c.as_str())
        );
        assert_eq!(ctx.execution.call_source, ctx2.execution.call_source);
    }

    #[test]
    fn test_inject_headers_includes_all_fields() {
        let ctx = RequestContext::new(
            SessionId::new("sess_123".to_string()),
            TraceId::new("trace_456".to_string()),
            ContextId::new("ctx_789".to_string()),
            AgentName::new("test-agent".to_string()),
        )
        .with_user_id(UserId::new("user_123".to_string()))
        .with_task_id(TaskId::new("task_456".to_string()));

        let mut headers = HeaderMap::new();
        ctx.inject_headers(&mut headers);

        assert!(headers.contains_key("x-session-id"));
        assert!(headers.contains_key("x-trace-id"));
        assert!(headers.contains_key("x-user-id"));
        assert!(headers.contains_key("x-context-id"));
        assert!(headers.contains_key("x-agent-name"));
        assert!(headers.contains_key("x-task-id"));
    }

    #[test]
    fn test_empty_context_id_not_injected() {
        let ctx = RequestContext::new(
            SessionId::new("sess_123".to_string()),
            TraceId::new("trace_456".to_string()),
            ContextId::new(String::new()),
            AgentName::new("test-agent".to_string()),
        );

        let mut headers = HeaderMap::new();
        ctx.inject_headers(&mut headers);

        assert!(!headers.contains_key("x-context-id"));
    }

    #[test]
    fn test_accessor_methods() {
        let ctx = RequestContext::new(
            SessionId::new("sess_123".to_string()),
            TraceId::new("trace_456".to_string()),
            ContextId::new("ctx_789".to_string()),
            AgentName::new("test-agent".to_string()),
        )
        .with_user_id(UserId::new("user_123".to_string()));

        assert_eq!(ctx.session_id().as_str(), "sess_123");
        assert_eq!(ctx.trace_id().as_str(), "trace_456");
        assert_eq!(ctx.user_id().as_str(), "user_123");
        assert_eq!(ctx.context_id().as_str(), "ctx_789");
        assert_eq!(ctx.agent_name().as_str(), "test-agent");
    }

    #[test]
    fn test_validation_methods() {
        let ctx_without_task = RequestContext::new(
            SessionId::new("sess_123".to_string()),
            TraceId::new("trace_456".to_string()),
            ContextId::new("ctx_789".to_string()),
            AgentName::new("test-agent".to_string()),
        );

        assert!(ctx_without_task.validate_task_execution().is_err());

        let ctx_with_task = ctx_without_task
            .clone()
            .with_task_id(TaskId::new("task_123".to_string()));

        assert!(ctx_with_task.validate_task_execution().is_ok());
    }

    #[test]
    fn test_component_separation() {
        let ctx = RequestContext::new(
            SessionId::new("sess_123".to_string()),
            TraceId::new("trace_456".to_string()),
            ContextId::new("ctx_789".to_string()),
            AgentName::new("test-agent".to_string()),
        )
        .with_user_id(UserId::new("user_123".to_string()))
        .with_user_type(UserType::Standard)
        .with_client_id(ClientId::new("client_123".to_string()))
        .with_task_id(TaskId::new("task_456".to_string()));

        assert_eq!(ctx.auth.user_id.as_str(), "user_123");
        assert_eq!(ctx.auth.user_type, UserType::Standard);
        assert_eq!(ctx.request.session_id.as_str(), "sess_123");
        assert_eq!(
            ctx.request.client_id.as_ref().unwrap().as_str(),
            "client_123"
        );
        assert_eq!(ctx.execution.trace_id.as_str(), "trace_456");
        assert_eq!(ctx.execution.context_id.as_str(), "ctx_789");
        assert_eq!(ctx.execution.agent_name.as_str(), "test-agent");
        assert_eq!(ctx.execution.task_id.as_ref().unwrap().as_str(), "task_456");
    }
}
