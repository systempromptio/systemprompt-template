use super::tools::{CallToolResult, ToolCall};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub request_id: Uuid,
    pub content: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: Option<u32>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooledResponse {
    pub request_id: Uuid,
    pub content: String,
    pub provider: String,
    pub model: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<CallToolResult>,
    pub tokens_used: Option<u32>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResponse {
    pub request_id: Uuid,
    pub content: String,
    pub provider: String,
    pub model: String,
    pub finish_reason: Option<String>,
    pub tokens_used: Option<u32>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cache_hit: bool,
    pub cache_read_tokens: Option<u32>,
    pub cache_creation_tokens: Option<u32>,
    pub is_streaming: bool,
    pub latency_ms: u64,
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
