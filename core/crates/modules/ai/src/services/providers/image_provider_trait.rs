use crate::errors::Result;
use crate::models::image_generation::{
    AspectRatio, ImageGenerationRequest, ImageGenerationResponse, ImageResolution,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Capabilities supported by an image provider
#[derive(Debug, Clone)]
pub struct ImageProviderCapabilities {
    /// Supported resolutions (e.g., 1K, 2K, 4K)
    pub supported_resolutions: Vec<ImageResolution>,

    /// Supported aspect ratios (e.g., 1:1, 16:9)
    pub supported_aspect_ratios: Vec<AspectRatio>,

    /// Whether the provider supports batch generation
    pub supports_batch: bool,

    /// Whether the provider supports image editing/reference images
    pub supports_image_editing: bool,

    /// Whether the provider supports Google Search grounding
    pub supports_search_grounding: bool,

    /// Maximum prompt length in characters
    pub max_prompt_length: usize,

    /// Cost per image in cents
    pub cost_per_image_cents: f32,
}

/// Trait for image generation providers
#[async_trait]
pub trait ImageProvider: Send + Sync {
    /// Get the provider name (e.g., "gemini", "openai-dalle", "stability-ai")
    fn name(&self) -> &str;

    /// Get the provider's capabilities
    fn capabilities(&self) -> ImageProviderCapabilities;

    /// Get supported models for this provider
    fn supported_models(&self) -> Vec<String>;

    /// Check if the provider supports a specific model
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().iter().any(|m| m == model)
    }

    /// Get the default model for this provider
    fn default_model(&self) -> &str;

    /// Check if a resolution is supported
    fn supports_resolution(&self, resolution: &ImageResolution) -> bool {
        self.capabilities()
            .supported_resolutions
            .contains(resolution)
    }

    /// Check if an aspect ratio is supported
    fn supports_aspect_ratio(&self, aspect_ratio: &AspectRatio) -> bool {
        self.capabilities()
            .supported_aspect_ratios
            .contains(aspect_ratio)
    }

    /// Generate a single image from a text prompt
    ///
    /// # Arguments
    /// * `request` - Image generation request with prompt and configuration
    ///
    /// # Returns
    /// * `ImageGenerationResponse` with base64-encoded image data and metadata
    async fn generate_image(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse>;

    /// Generate multiple images in a batch (if supported)
    ///
    /// # Arguments
    /// * `requests` - Multiple image generation requests
    ///
    /// # Returns
    /// * Vec of responses, one for each request
    async fn generate_batch(
        &self,
        requests: &[ImageGenerationRequest],
    ) -> Result<Vec<ImageGenerationResponse>> {
        if !self.capabilities().supports_batch {
            return Err(crate::errors::AiError::ProviderError {
                provider: self.name().to_string(),
                message: "Batch generation not supported by this provider".to_string(),
            });
        }

        // Default implementation: generate one by one
        let mut responses = Vec::new();
        for request in requests {
            responses.push(self.generate_image(request).await?);
        }
        Ok(responses)
    }
}

/// Type alias for boxed image providers
pub type BoxedImageProvider = Arc<dyn ImageProvider>;
