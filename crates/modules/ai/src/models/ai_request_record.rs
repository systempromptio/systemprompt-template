use systemprompt_identifiers::{ContextId, McpExecutionId, SessionId, TaskId, TraceId, UserId};

#[derive(Debug, Clone, Copy, Default)]
pub struct TokenInfo {
    pub tokens_used: Option<i32>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheInfo {
    pub cache_hit: bool,
    pub cache_read_tokens: Option<i32>,
    pub cache_creation_tokens: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    Pending,
    Completed,
    Failed,
}

impl RequestStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiRequestRecord {
    pub request_id: String,
    pub user_id: UserId,
    pub session_id: Option<SessionId>,
    pub task_id: Option<TaskId>,
    pub context_id: Option<ContextId>,
    pub trace_id: Option<TraceId>,
    pub mcp_execution_id: Option<McpExecutionId>,
    pub provider: String,
    pub model: String,
    pub tokens: TokenInfo,
    pub cache: CacheInfo,
    pub is_streaming: bool,
    pub cost_cents: i32,
    pub latency_ms: i32,
    pub status: RequestStatus,
    pub error_message: Option<String>,
}

impl AiRequestRecord {
    pub fn builder(request_id: impl Into<String>, user_id: UserId) -> AiRequestRecordBuilder {
        AiRequestRecordBuilder::new(request_id, user_id)
    }
}

#[derive(Debug)]
pub struct AiRequestRecordBuilder {
    request_id: String,
    user_id: UserId,
    session_id: Option<SessionId>,
    task_id: Option<TaskId>,
    context_id: Option<ContextId>,
    trace_id: Option<TraceId>,
    mcp_execution_id: Option<McpExecutionId>,
    provider: Option<String>,
    model: Option<String>,
    tokens: TokenInfo,
    cache: CacheInfo,
    is_streaming: bool,
    cost_cents: i32,
    latency_ms: i32,
    status: RequestStatus,
    error_message: Option<String>,
}

impl AiRequestRecordBuilder {
    pub fn new(request_id: impl Into<String>, user_id: UserId) -> Self {
        Self {
            request_id: request_id.into(),
            user_id,
            session_id: None,
            task_id: None,
            context_id: None,
            trace_id: None,
            mcp_execution_id: None,
            provider: None,
            model: None,
            tokens: TokenInfo::default(),
            cache: CacheInfo::default(),
            is_streaming: false,
            cost_cents: 0,
            latency_ms: 0,
            status: RequestStatus::Pending,
            error_message: None,
        }
    }

    pub fn session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn task_id(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn context_id(mut self, context_id: ContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }

    pub fn trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn mcp_execution_id(mut self, mcp_execution_id: McpExecutionId) -> Self {
        self.mcp_execution_id = Some(mcp_execution_id);
        self
    }

    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub const fn tokens(mut self, input: Option<i32>, output: Option<i32>) -> Self {
        self.tokens.input_tokens = input;
        self.tokens.output_tokens = output;
        self.tokens.tokens_used = match (input, output) {
            (Some(i), Some(o)) => Some(i + o),
            (Some(i), None) => Some(i),
            (None, Some(o)) => Some(o),
            (None, None) => None,
        };
        self
    }

    pub const fn cache(
        mut self,
        hit: bool,
        read_tokens: Option<i32>,
        creation_tokens: Option<i32>,
    ) -> Self {
        self.cache.cache_hit = hit;
        self.cache.cache_read_tokens = read_tokens;
        self.cache.cache_creation_tokens = creation_tokens;
        self
    }

    pub const fn streaming(mut self, is_streaming: bool) -> Self {
        self.is_streaming = is_streaming;
        self
    }

    pub const fn cost(mut self, cost_cents: i32) -> Self {
        self.cost_cents = cost_cents;
        self
    }

    pub const fn latency(mut self, latency_ms: i32) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    pub const fn completed(mut self) -> Self {
        self.status = RequestStatus::Completed;
        self
    }

    pub fn failed(mut self, error_message: impl Into<String>) -> Self {
        self.status = RequestStatus::Failed;
        self.error_message = Some(error_message.into());
        self
    }

    pub fn build(self) -> Result<AiRequestRecord, AiRequestRecordError> {
        let provider = self.provider.ok_or(AiRequestRecordError::MissingProvider)?;
        let model = self.model.ok_or(AiRequestRecordError::MissingModel)?;

        Ok(AiRequestRecord {
            request_id: self.request_id,
            user_id: self.user_id,
            session_id: self.session_id,
            task_id: self.task_id,
            context_id: self.context_id,
            trace_id: self.trace_id,
            mcp_execution_id: self.mcp_execution_id,
            provider,
            model,
            tokens: self.tokens,
            cache: self.cache,
            is_streaming: self.is_streaming,
            cost_cents: self.cost_cents,
            latency_ms: self.latency_ms,
            status: self.status,
            error_message: self.error_message,
        })
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum AiRequestRecordError {
    #[error("Provider is required")]
    MissingProvider,
    #[error("Model is required")]
    MissingModel,
}
