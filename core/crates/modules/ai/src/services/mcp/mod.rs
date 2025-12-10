//! MCP Tool Execution Module
//!
//! Execute MCP tools during AI conversations and agentic workflows.
//!
//! ## Purpose
//!
//! This module provides runtime tool execution for AI services. When an LLM
//! decides to use a tool during conversation, this module:
//! - Connects to running MCP servers
//! - Discovers available tools
//! - Executes tool calls from LLM responses
//! - Returns execution results
//!
//! ## Critical Distinction
//!
//! **AI MCP** (this module): Execute tools during conversations → `ToolResult`
//! **Agent MCP** (`agent/services/external_integrations/mcp`): Load skills for
//! discovery → `AgentSkill`
//!
//! ## Components
//!
//! - [`McpClientManager`] - Connection pooling and service management
//! - [`ToolDiscovery`] - Tool discovery and filtering

pub mod client;
pub mod tool_discovery;

pub use client::McpClientManager;
pub use tool_discovery::ToolDiscovery;
