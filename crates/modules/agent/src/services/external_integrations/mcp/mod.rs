//! MCP (Model Context Protocol) Integration Module
//!
//! This module handles dynamic tool loading from MCP servers for A2A agents.
//! Tools are fetched on-demand from running MCP services and included in agent
//! card extensions.
//!
//! ## Architecture
//!
//! 1. Agent assigns MCP servers via config metadata (mcp_servers array)
//! 2. Query services table for MCP server state (port, status)
//! 3. Connect to running MCP servers via HTTP
//! 4. Fetch tools using rmcp protocol
//! 5. Include tools in AgentCard extension metadata (systemprompt:mcp-tools)
//! 6. Skills reference tools via allowed-tools field in SKILL.md
//!
//! ## Usage
//!
//! ```rust
//! use crate::services::external_integrations::mcp::McpToolLoader;
//!
//! let loader = McpToolLoader::new(db_pool);
//! let tools_by_server = loader.load_tools_for_servers(&server_names).await?;
//! let extensions = loader
//!     .create_mcp_extensions(&server_names, base_url)
//!     .await?;
//! ```

pub mod client;
pub mod models;
pub mod orchestration;
pub mod service;

pub use client::McpClientAdapter;
pub use models::*;
pub use orchestration::McpToolLoader;
pub use service::{McpServiceState, ServiceStateManager};

pub use systemprompt_core_mcp::services::client::McpClient;
pub use systemprompt_models::ai::tools::McpTool;
