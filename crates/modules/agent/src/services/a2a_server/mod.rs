//! A2A Protocol Server
//!
//! HTTP server that receives and processes A2A protocol requests.
//!
//! ## Module Organization
//!
//! - `server` - HTTP server setup and lifecycle
//! - `handlers/` - Request handling layer
//!   - `request` - Main A2A request handler
//!   - `card` - Agent card endpoint
//!   - `state` - Shared handler state
//! - `processing/` - Business logic layer
//!   - `message` - Message processing orchestration
//!   - `ai_executor` - AI service integration
//!   - `artifact` - Artifact building from tool results
//! - `streaming/` - SSE streaming layer
//!   - `messages` - Event-based message streaming (real-time)
//! - `auth/` - Authentication & authorization
//!   - `middleware` - Axum middleware
//!   - `validation` - Token validation
//!   - `types` - Auth types
//! - `config/` - Configuration
//!   - `agent` - Agent configuration logic
//!   - `types` - Config types
//! - `builders/` - Data construction
//!   - `task` - Task builder pattern
//! - `errors/` - Error handling
//!   - `jsonrpc` - JSON-RPC error builder

pub mod auth;
pub mod builders;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod processing;
pub mod server;
pub mod streaming;

// Public API - only export what's used outside this module
pub use config::AgentConfig;
pub use handlers::AgentHandlerState;
pub use server::Server;
