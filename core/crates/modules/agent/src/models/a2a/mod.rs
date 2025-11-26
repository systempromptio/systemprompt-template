//! A2A Protocol Domain Models
//!
//! Core domain entities for A2A (Agent-to-Agent) protocol specification.

pub mod agent;
pub mod artifact;
pub mod auth;
pub mod jsonrpc;
pub mod mcp_extension;
pub mod message;
pub mod protocol;
pub mod task;
pub mod transport;

pub use agent::{
    AgentCapabilities, AgentCard, AgentCardSignature, AgentExtension, AgentInterface,
    AgentProvider, AgentSkill,
};
pub use artifact::Artifact;
pub use auth::{AgentAuthentication, ApiKeyLocation, OAuth2Flow, OAuth2Flows, SecurityScheme};
pub use mcp_extension::McpServerMetadata;
pub use message::{DataPart, FilePart, FileWithBytes, Message, MessageRole, Part, TextPart};
pub use protocol::{
    A2aJsonRpcRequest, A2aParseError, A2aRequest, A2aRequestParams, A2aResponse, MessageSendParams,
    TaskIdParams, TaskQueryParams,
};
pub use task::{Task, TaskState, TaskStatus};
pub use transport::TransportProtocol;
