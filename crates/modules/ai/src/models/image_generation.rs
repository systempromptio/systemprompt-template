use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported image resolutions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageResolution {
    #[serde(rename = "1K")]
    OneK,
    #[serde(rename = "2K")]
    TwoK,
    #[serde(rename = "4K")]
    FourK,
}

impl ImageResolution {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::OneK => "1K",
            Self::TwoK => "2K",
            Self::FourK => "4K",
        }
    }
}

impl Default for ImageResolution {
    fn default() -> Self {
        Self::OneK
    }
}

/// Supported aspect ratios
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectRatio {
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "16:9")]
    Landscape169,
    #[serde(rename = "9:16")]
    Portrait916,
    #[serde(rename = "4:3")]
    Landscape43,
    #[serde(rename = "3:4")]
    Portrait34,
    #[serde(rename = "21:9")]
    UltraWide,
}

impl AspectRatio {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Square => "1:1",
            Self::Landscape169 => "16:9",
            Self::Portrait916 => "9:16",
            Self::Landscape43 => "4:3",
            Self::Portrait34 => "3:4",
            Self::UltraWide => "21:9",
        }
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::Square
    }
}

/// Request for generating an image from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Text prompt describing the image to generate
    pub prompt: String,

    /// Optional specific model to use (if not provided, uses provider default)
    pub model: Option<String>,

    /// Desired resolution (default: 1K)
    #[serde(default)]
    pub resolution: ImageResolution,

    /// Desired aspect ratio (default: 1:1)
    #[serde(default)]
    pub aspect_ratio: AspectRatio,

    /// Optional reference images for editing/style transfer (base64-encoded)
    #[serde(default)]
    pub reference_images: Vec<ReferenceImage>,

    /// Enable Google Search grounding (if supported by provider)
    #[serde(default)]
    pub enable_search_grounding: bool,

    /// User context for tracking and analytics
    #[serde(default)]
    pub user_id: Option<String>,

    #[serde(default)]
    pub session_id: Option<String>,

    #[serde(default)]
    pub trace_id: Option<String>,
}

/// Reference image for editing or style transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceImage {
    /// Base64-encoded image data
    pub data: String,

    /// MIME type (e.g., "image/png")
    pub mime_type: String,

    /// Optional description of how to use this image
    pub description: Option<String>,
}

/// Response from image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Unique identifier for this generation
    pub id: String,

    /// Request ID linking to `ai_requests` table
    pub request_id: String,

    /// Provider that generated the image
    pub provider: String,

    /// Model used for generation
    pub model: String,

    /// Base64-encoded image data
    pub image_data: String,

    /// MIME type of the generated image
    pub mime_type: String,

    /// File path where image is stored (after save)
    pub file_path: Option<String>,

    /// Public URL for accessing the image (after save)
    pub public_url: Option<String>,

    /// Size of the image in bytes
    pub file_size_bytes: Option<usize>,

    /// Generation configuration
    pub resolution: ImageResolution,
    pub aspect_ratio: AspectRatio,

    /// Performance metrics
    pub generation_time_ms: u64,

    /// Cost estimate in cents
    pub cost_estimate: Option<f32>,

    /// Timestamp when generated
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ImageGenerationResponse {
    pub fn new(
        provider: String,
        model: String,
        image_data: String,
        mime_type: String,
        resolution: ImageResolution,
        aspect_ratio: AspectRatio,
        generation_time_ms: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            request_id: Uuid::new_v4().to_string(),
            provider,
            model,
            image_data,
            mime_type,
            file_path: None,
            public_url: None,
            file_size_bytes: None,
            resolution,
            aspect_ratio,
            generation_time_ms,
            cost_estimate: None,
            created_at: chrono::Utc::now(),
        }
    }
}

/// Stored image metadata (for database persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImageRecord {
    pub uuid: String,
    pub request_id: String,
    pub prompt: String,
    pub model: String,
    pub provider: String,
    pub file_path: String,
    pub public_url: String,
    pub file_size_bytes: Option<i32>,
    pub mime_type: String,
    pub resolution: Option<String>,
    pub aspect_ratio: Option<String>,
    pub generation_time_ms: Option<i32>,
    pub cost_estimate: Option<f32>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
