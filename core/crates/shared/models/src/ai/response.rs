use super::tools::{CallToolResult, ToolCall};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub request_id: Uuid,
    pub content: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: Option<u32>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub latency_ms: u64,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<CallToolResult>,
    pub finish_reason: Option<String>,
    pub cache_hit: bool,
    pub cache_read_tokens: Option<u32>,
    pub cache_creation_tokens: Option<u32>,
    pub is_streaming: bool,
}

impl Default for AiResponse {
    fn default() -> Self {
        Self {
            request_id: Uuid::nil(),
            content: String::new(),
            provider: String::new(),
            model: String::new(),
            tokens_used: None,
            input_tokens: None,
            output_tokens: None,
            latency_ms: 0,
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            finish_reason: None,
            cache_hit: false,
            cache_read_tokens: None,
            cache_creation_tokens: None,
            is_streaming: false,
        }
    }
}

impl AiResponse {
    pub fn new(request_id: Uuid, content: String, provider: String, model: String) -> Self {
        Self {
            request_id,
            content,
            provider,
            model,
            ..Default::default()
        }
    }

    pub const fn with_tokens(mut self, tokens_used: u32) -> Self {
        self.tokens_used = Some(tokens_used);
        self
    }

    pub const fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = tool_calls;
        self
    }

    pub fn with_tool_results(mut self, tool_results: Vec<CallToolResult>) -> Self {
        self.tool_results = tool_results;
        self
    }

    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    pub fn has_tool_results(&self) -> bool {
        !self.tool_results.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSource {
    pub title: String,
    pub uri: String,
    pub relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlMetadata {
    pub retrieved_url: String,
    pub url_retrieval_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchGroundedResponse {
    pub content: String,
    pub sources: Vec<WebSource>,
    pub confidence_scores: Vec<f32>,
    pub web_search_queries: Vec<String>,
    pub url_context_metadata: Option<Vec<UrlMetadata>>,
    pub tokens_used: Option<u32>,
    pub latency_ms: u64,
    pub finish_reason: Option<String>,
    pub safety_ratings: Option<Vec<serde_json::Value>>,
}
