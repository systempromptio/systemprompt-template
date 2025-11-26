//! SystemPrompt Agent Module - Modern Architecture
//!
//! This module provides Agent-to-Agent protocol implementation with clean, modern architecture.
//! Implements the new world-class module system with proper trait separation.

pub mod api;
pub mod errors;
pub mod models;
pub mod queries;
pub mod repository;
pub mod services;
// pub mod client;
pub mod utils;

// Export A2A domain models
pub use models::a2a::{
    // Request/Response types
    A2aJsonRpcRequest,
    A2aRequestParams,
    A2aResponse,
    AgentCapabilities,
    AgentCard,
    AgentInterface,
    AgentProvider,
    AgentSkill,
    Artifact,
    DataPart,
    Message,
    MessageSendParams,
    Part,
    SecurityScheme,
    // Core types
    Task,
    TaskIdParams,
    // Client types
    // A2aClient, ClientConfig, // Removed during restructuring
    TaskQueryParams,
    TaskState,
    TaskStatus,
    TextPart,
    TransportProtocol,
};

// Export error types
pub use errors::{AgentError, ArtifactError, ContextError, ProtocolError, TaskError};

// Export services and skills
pub use services::{
    AgentHandlerState, AgentOrchestrator, AgentServer, AgentStatus, ContextService,
    SkillIngestionService, SkillService,
};
