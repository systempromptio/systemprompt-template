//! Agent Models
//!
//! Domain models organized by concern:
//! - a2a: A2A protocol specification types
//! - web: REST API models
//! - external_integrations: External integration types

pub mod a2a;
pub mod agent_info;
pub mod context;
pub mod database_rows;
pub mod execution_step;
pub mod external_integrations;
pub mod runtime;
pub mod skill;
pub mod web;

// Re-export commonly used A2A types
pub use a2a::{
    AgentAuthentication, AgentCapabilities, AgentCard, AgentSkill, Artifact, DataPart, Message,
    Part, Task, TaskState, TaskStatus, TextPart, TransportProtocol,
};

// Re-export agent info
pub use agent_info::AgentInfo;

// Re-export runtime types
pub use runtime::AgentRuntimeInfo;

// Re-export context types
pub use context::{
    ContextDetail, ContextMessage, CreateContextRequest, UpdateContextRequest, UserContext,
    UserContextWithStats,
};

// Re-export skill types
pub use skill::{Skill, SkillMetadata};

// Re-export execution step types
pub use execution_step::{
    ExecutionStep, PlannedTool, StepContent, StepId, StepStatus, StepType, TrackedStep,
};

// Re-export database row types
pub use database_rows::{
    ArtifactPartRow, ArtifactRow, ExecutionStepBatchRow, MessagePart, PushNotificationConfigRow,
    SkillRow, TaskMessage, TaskRow,
};

pub use web::*;
