//! Services for A2A protocol operations.
//!
//! Currently implements:
//! - `a2a_server`: HTTP server for A2A protocol
//! - `a2a_client`: HTTP client for A2A protocol
//! - `agent_orchestration`: Database-driven agent management
//! - `external_integrations`: OAuth, MCP, and webhook services
//! - `mcp`: MCP tool artifact transformation
//! - `shared`: Common utilities and traits

pub mod a2a_client;
pub mod a2a_server;
pub mod agent_orchestration;
pub mod artifact_publishing_service;
pub mod context_service;
pub mod execution_tracking_service;
pub mod external_integrations;
pub mod mcp;
pub mod message_service;
pub mod registry;
pub mod shared;
pub mod skills;
pub mod slugify;

// Re-export server types
pub use a2a_server::{AgentHandlerState, Server as AgentServer};

// Re-export client types
pub use a2a_client::{A2aClient, ClientConfig, ClientError, ClientResult};

// Re-export agent orchestration types
pub use agent_orchestration::{
    AgentOrchestrator, AgentStatus, OrchestrationError, OrchestrationResult,
};

// Re-export registry types
pub use registry::AgentRegistry;

// Re-export external integrations types
pub use external_integrations::{
    IntegrationError, IntegrationResult, McpServiceState, McpToolLoader, RegisteredMcpServer,
    ServiceStateManager, ToolExecutionResult, WebhookEndpoint, WebhookService,
};

// Re-export skills types
pub use skills::{SkillIngestionService, SkillMetadata, SkillService};

// Re-export artifact publishing service
pub use artifact_publishing_service::ArtifactPublishingService;

// Re-export message service
pub use message_service::MessageService;

// Re-export context service
pub use context_service::ContextService;

// Re-export execution tracking service
pub use execution_tracking_service::ExecutionTrackingService;

// Re-export slugify utilities
pub use slugify::{generate_slug, generate_unique_slug};
