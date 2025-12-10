#![allow(clippy::all)]
#![allow(clippy::pedantic)]

pub mod api;
pub mod error;
pub mod models;
pub mod repository;
pub mod services;

pub use models::a2a::{
    A2aJsonRpcRequest, A2aRequestParams, A2aResponse, AgentCapabilities, AgentCard, AgentInterface,
    AgentProvider, AgentSkill, Artifact, DataPart, Message, MessageSendParams, Part,
    SecurityScheme, Task, TaskIdParams, TaskQueryParams, TaskState, TaskStatus, TextPart,
    TransportProtocol,
};

pub use error::{AgentError, ArtifactError, ContextError, ProtocolError, TaskError};

pub use services::{
    AgentHandlerState, AgentOrchestrator, AgentServer, AgentStatus, ContextService,
    SkillIngestionService, SkillService,
};
