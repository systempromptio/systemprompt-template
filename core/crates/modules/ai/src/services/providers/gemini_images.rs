use crate::errors::{AiError, Result};
use crate::models::image_generation::{
    AspectRatio, ImageGenerationRequest, ImageGenerationResponse, ImageResolution,
};
use crate::models::providers::gemini::{
    GeminiContent, GeminiGenerationConfig, GeminiInlineData, GeminiPart,
    GeminiRequest, GeminiResponse, GeminiTool, GoogleSearch,
};
use crate::services::providers::image_provider_trait::{ImageProvider, ImageProviderCapabilities};
use async_trait::async_trait;
use reqwest::Client;
use std::time::Instant;

pub struct GeminiImageProvider {
    client: Client,
    api_key: String,
    endpoint: String,
    default_model: String,
}

impl GeminiImageProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            api_key,
            endpoint: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            default_model: "gemini-2.5-flash-image".to_string(),
        }
    }

    pub fn with_endpoint(api_key: String, endpoint: String) -> Self {
        let mut provider = Self::new(api_key);
        provider.endpoint = endpoint;
        provider
    }

    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = model;
        self
    }

    fn build_image_request(request: &ImageGenerationRequest) -> GeminiRequest {
        let mut parts = vec![GeminiPart::Text {
            text: request.prompt.clone(),
        }];

        // Add reference images if provided
        for ref_image in &request.reference_images {
            parts.push(GeminiPart::InlineData {
                inline_data: GeminiInlineData {
                    mime_type: ref_image.mime_type.clone(),
                    data: ref_image.data.clone(),
                },
            });
            if let Some(desc) = &ref_image.description {
                parts.push(GeminiPart::Text { text: desc.clone() });
            }
        }

        let contents = vec![GeminiContent {
            role: "user".to_string(),
            parts,
        }];

        // NOTE: imageConfig causes 500 errors from Gemini API
        // Gemini will use default size/aspect ratio when not specified
        let generation_config = GeminiGenerationConfig {
            temperature: None,
            top_p: None,
            top_k: None,
            max_output_tokens: None,
            stop_sequences: None,
            response_mime_type: None,
            response_schema: None,
            response_modalities: Some(vec!["IMAGE".to_string()]),
            image_config: None,
        };

        let tools = if request.enable_search_grounding {
            Some(vec![GeminiTool {
                function_declarations: None,
                google_search: Some(GoogleSearch {}),
                url_context: None,
            }])
        } else {
            None
        };

        GeminiRequest {
            contents,
            generation_config: Some(generation_config),
            safety_settings: None,
            tools,
        }
    }

    fn extract_image_from_response(response: &GeminiResponse) -> Result<(String, String)> {
        let candidate =
            response
                .candidates
                .first()
                .ok_or_else(|| AiError::EmptyProviderResponse {
                    provider: "gemini-image".to_string(),
                })?;

        let content = candidate
            .content
            .as_ref()
            .ok_or_else(|| AiError::ProviderError {
                provider: "gemini-image".to_string(),
                message: "No content in response".to_string(),
            })?;

        for part in &content.parts {
            if let GeminiPart::InlineData { inline_data } = part {
                return Ok((inline_data.data.clone(), inline_data.mime_type.clone()));
            }
        }

        Err(AiError::ProviderError {
            provider: "gemini-image".to_string(),
            message: "No image data found in response".to_string(),
        })
    }
}

#[async_trait]
impl ImageProvider for GeminiImageProvider {
    fn name(&self) -> &'static str {
        "gemini-image"
    }

    fn capabilities(&self) -> ImageProviderCapabilities {
        ImageProviderCapabilities {
            supported_resolutions: vec![
                ImageResolution::OneK,
                ImageResolution::TwoK,
                ImageResolution::FourK,
            ],
            supported_aspect_ratios: vec![
                AspectRatio::Square,
                AspectRatio::Landscape169,
                AspectRatio::Portrait916,
                AspectRatio::Landscape43,
                AspectRatio::Portrait34,
                AspectRatio::UltraWide,
            ],
            supports_batch: true,
            supports_image_editing: true,
            supports_search_grounding: true,
            max_prompt_length: 8000,
            cost_per_image_cents: 0.04, // $0.04 per image (Gemini 2.5 Flash Image pricing)
        }
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gemini-2.5-flash-image".to_string(),
            "gemini-3-pro-image-preview".to_string(),
        ]
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn generate_image(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse> {
        let start = Instant::now();

        // Validate prompt length
        if request.prompt.len() > self.capabilities().max_prompt_length {
            return Err(AiError::ProviderError {
                provider: self.name().to_string(),
                message: format!(
                    "Prompt length {} exceeds maximum {}",
                    request.prompt.len(),
                    self.capabilities().max_prompt_length
                ),
            });
        }

        // Validate resolution
        if !self.supports_resolution(&request.resolution) {
            return Err(AiError::ProviderError {
                provider: self.name().to_string(),
                message: format!("Resolution {} not supported", request.resolution.as_str()),
            });
        }

        // Validate aspect ratio
        if !self.supports_aspect_ratio(&request.aspect_ratio) {
            return Err(AiError::ProviderError {
                provider: self.name().to_string(),
                message: format!(
                    "Aspect ratio {} not supported",
                    request.aspect_ratio.as_str()
                ),
            });
        }

        // Determine model
        let model = request
            .model.as_deref()
            .unwrap_or(self.default_model());

        if !self.supports_model(model) {
            return Err(AiError::ProviderError {
                provider: self.name().to_string(),
                message: format!("Model {model} not supported"),
            });
        }

        // Build request
        let gemini_request = Self::build_image_request(request);

        // Make API call
        let url = format!("{}/models/{}:generateContent", self.endpoint, model);

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| AiError::ProviderError {
                provider: self.name().to_string(),
                message: format!("HTTP request failed: {e}"),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: self.name().to_string(),
                message: format!("API returned status {status}: {error_body}"),
            });
        }

        let gemini_response: GeminiResponse =
            response.json().await.map_err(|e| AiError::ProviderError {
                provider: self.name().to_string(),
                message: format!("Failed to parse response: {e}"),
            })?;

        // Extract image data
        let (image_data, mime_type) = Self::extract_image_from_response(&gemini_response)?;

        let generation_time_ms = start.elapsed().as_millis() as u64;

        Ok(ImageGenerationResponse::new(
            self.name().to_string(),
            model.to_string(),
            image_data,
            mime_type,
            request.resolution.clone(),
            request.aspect_ratio.clone(),
            generation_time_ms,
        ))
    }

    async fn generate_batch(
        &self,
        requests: &[ImageGenerationRequest],
    ) -> Result<Vec<ImageGenerationResponse>> {
        // Gemini supports batch generation via the Batch API
        // For now, implement sequential generation
        // TODO: Implement true batch API support

        let mut responses = Vec::new();
        for request in requests {
            responses.push(self.generate_image(request).await?);
        }
        Ok(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities() {
        let api_key = "test_key".to_string();
        let provider = GeminiImageProvider::new(api_key);

        let caps = provider.capabilities();
        assert_eq!(caps.supported_resolutions.len(), 3);
        assert_eq!(caps.supported_aspect_ratios.len(), 6);
        assert!(caps.supports_batch);
        assert!(caps.supports_image_editing);
        assert!(caps.supports_search_grounding);
    }

    #[test]
    fn test_supported_models() {
        let api_key = "test_key".to_string();
        let provider = GeminiImageProvider::new(api_key);

        let models = provider.supported_models();
        assert_eq!(models.len(), 2);
        assert!(provider.supports_model("gemini-2.5-flash-image"));
        assert!(provider.supports_model("gemini-3-pro-image-preview"));
        assert!(!provider.supports_model("gpt-4"));
    }
}
