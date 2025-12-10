//! External Integrations Service
//!
//! Provides unified access to OAuth providers, MCP servers, and webhook
//! endpoints. Designed for simplicity and maintainability with clear separation
//! of concerns.
//!
//! ## Core Services
//!
//! - **OAuth Service**: Manages OAuth2 providers and token flows
//! - **MCP Service**: Handles MCP server discovery and tool execution
//! - **Webhook Service**: Manages webhook endpoints and signature validation
//! - **Integration Manager**: Orchestrates all external integration operations
//!
//! ## Usage
//!
//! ```rust
//! use crate::services::external_integrations::IntegrationManager;
//!
//! let manager = IntegrationManager::new(db_pool);
//!
//! // OAuth operations
//! let auth_request = manager
//!     .start_oauth_flow("github", "agent-123", vec!["repo".to_string()])
//!     .await?;
//!
//! // MCP operations
//! let tools = manager.get_agent_capabilities("agent-123").await?;
//! let result = manager
//!     .execute_agent_tool("agent-123", "database_query", arguments)
//!     .await?;
//!
//! // Webhook operations
//! manager
//!     .notify_external_service("https://api.example.com/webhook", event_data)
//!     .await?;
//! ```

pub mod mcp;
pub mod webhook;

pub use crate::models::external_integrations::{
    IntegrationError, IntegrationResult, RegisteredMcpServer, ToolExecutionResult, WebhookEndpoint,
    WebhookRequest, WebhookResponse,
};

pub use mcp::{McpClient, McpServiceState, McpTool, McpToolLoader, ServiceStateManager};
pub use webhook::{RetryPolicy, WebhookConfig, WebhookDeliveryResult, WebhookService};
