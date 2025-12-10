#![allow(clippy::pedantic)]
#![allow(clippy::too_many_arguments)]

pub mod error;
pub mod models;
pub mod repository;
pub mod services;

pub use error::{AiError, RepositoryError, Result};

pub use services::core::{AiService, ImageService};

pub use services::storage::{ImageStorage, StorageConfig};

pub use systemprompt_models::ai::{
    AiMessage, AiRequest, AiResponse, MessageRole, ModelConfig, ModelHint, ModelPreferences,
    ResponseFormat, SamplingMetadata, SearchGroundedResponse, StructuredOutputOptions,
};

pub use systemprompt_models::ai::tools::{CallToolResult, McpTool, ToolCall, ToolExecution};

pub use services::config::loader::AiConfig;

pub use services::schema::{
    ProviderCapabilities, SchemaTransformer, ToolNameMapper, TransformedTool,
};

pub use models::image_generation::{
    AspectRatio, GeneratedImageRecord, ImageGenerationRequest, ImageGenerationResponse,
    ImageResolution, ReferenceImage,
};

pub use services::providers::{GeminiImageProvider, ImageProvider, ImageProviderCapabilities};

pub use repository::{AIRequestRepository, ImageRepository};

pub use models::{
    AIRequest, AIRequestMessage, AIRequestToolCall, CostSummary, GeneratedImage,
    LatencyPercentiles, ProviderUsage, UserAIUsage,
};

pub use services::tooled::ToolResultFormatter;
