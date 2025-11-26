pub mod errors;
pub mod models;
pub mod repository;
pub mod services;
pub mod storage;

pub use errors::{AiError, Result};

// Primary service exports
pub use services::core::{AgenticExecutionResult, AgenticExecutor, AiService, ImageService};

// Storage exports
pub use storage::{ImageStorage, StorageConfig};

// Re-export from shared models
pub use systemprompt_models::ai::{
    AiMessage, GenerateRequest, GenerateResponse, MessageRole, ModelConfig, ModelHint,
    ModelPreferences, ResponseFormat, SamplingMetadata, SamplingRequest, SamplingResponse,
    StructuredOutputOptions, TooledRequest, TooledResponse,
};

// Tool types from shared models
pub use systemprompt_models::ai::tools::{CallToolResult, McpTool, ToolCall, ToolExecution};

// Configuration types
pub use services::config::loader::AiConfig;

// Schema transformation types
pub use services::schema::{
    ProviderCapabilities, SchemaTransformer, ToolNameMapper, TransformedTool,
};

// Image generation types
pub use models::image_generation::{
    AspectRatio, GeneratedImageRecord, ImageGenerationRequest, ImageGenerationResponse,
    ImageResolution, ReferenceImage,
};

// Image provider types
pub use services::providers::{GeminiImageProvider, ImageProvider, ImageProviderCapabilities};

// Repository exports
pub use repository::{AiRequestRepository, ImageRepository};

// Tooled execution utilities
pub use services::tooled::ToolResultFormatter;
