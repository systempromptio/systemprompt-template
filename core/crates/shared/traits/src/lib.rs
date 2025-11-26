pub mod artifact;
pub mod context;
pub mod module;
pub mod repository;
pub mod service;
pub mod validation;

// Re-export core traits
pub use context::{
    ApiModule, AppContext, ConfigProvider, ContextPropagation, DatabaseHandle,
    InjectContextHeaders, Module, ModuleRegistry,
};

// Re-export repository traits
pub use repository::{CrudRepository, Repository, RepositoryError};

// Re-export service traits
pub use service::{AsyncService, Service};

// Re-export artifact support
pub use artifact::{schemas, ArtifactSupport};

// Re-export validation traits
pub use validation::{MetadataValidation, Validate, ValidationError, ValidationResult};

// Generic Result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
