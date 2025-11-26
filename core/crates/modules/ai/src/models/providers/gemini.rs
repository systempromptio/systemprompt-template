use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiModels {
    pub gemini_flash_lite: ModelConfig,
    pub gemini_flash: ModelConfig,
}

pub use systemprompt_core_system::ModelConfig;

impl Default for GeminiModels {
    fn default() -> Self {
        Self {
            gemini_flash_lite: ModelConfig {
                id: "gemini-2.5-flash-lite".to_string(),
                max_tokens: 1_000_000,
                supports_tools: true,
                cost_per_1k_tokens: 0.00015,
            },
            gemini_flash: ModelConfig {
                id: "gemini-2.5-flash".to_string(),
                max_tokens: 1_000_000,
                supports_tools: true,
                cost_per_1k_tokens: 0.00025,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    pub generation_config: Option<GeminiGenerationConfig>,
    pub safety_settings: Option<Vec<GeminiSafetySetting>>,
    pub tools: Option<Vec<GeminiTool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiContent {
    pub role: String,
    #[serde(default)]
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GeminiPart {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: GeminiInlineData,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: GeminiFunctionResponse,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiInlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGenerationConfig {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub max_output_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_config: Option<GeminiImageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiImageConfig {
    pub aspect_ratio: String,
    pub image_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiSafetySetting {
    pub category: String,
    pub threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiTool {
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "functionDeclarations"
    )]
    pub function_declarations: Option<Vec<GeminiFunctionDeclaration>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "googleSearch")]
    pub google_search: Option<GoogleSearch>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "urlContext")]
    pub url_context: Option<UrlContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleSearch {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlContext {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
    pub usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiCandidate {
    pub content: Option<GeminiContent>,
    pub finish_reason: Option<String>,
    pub index: Option<i32>,
    pub safety_ratings: Option<Vec<GeminiSafetyRating>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GeminiGroundingMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context_metadata: Option<GeminiUrlContextMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiSafetyRating {
    pub category: String,
    pub probability: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct GeminiUsageMetadata {
    pub prompt_token_count: u32,
    #[serde(default)]
    pub candidates_token_count: Option<u32>,
    pub total_token_count: u32,
}

// Google Search grounding metadata (only present when googleSearch tool is used)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGroundingMetadata {
    #[serde(default)]
    pub grounding_chunks: Vec<GeminiGroundingChunk>,
    #[serde(default)]
    pub grounding_supports: Vec<GeminiGroundingSupport>,
    #[serde(default)]
    pub web_search_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiGroundingChunk {
    pub web: GeminiWebSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiWebSource {
    pub uri: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGroundingSupport {
    pub segment: GeminiTextSegment,
    pub grounding_chunk_indices: Vec<i32>,
    #[serde(default)]
    pub confidence_scores: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiTextSegment {
    pub start_index: i32,
    pub end_index: i32,
    pub text: String,
}

// URL context metadata (only present when urlContext tool is used)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiUrlContextMetadata {
    #[serde(default)]
    pub url_metadata: Vec<GeminiUrlMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiUrlMetadata {
    pub retrieved_url: String,
    pub url_retrieval_status: String,
}
