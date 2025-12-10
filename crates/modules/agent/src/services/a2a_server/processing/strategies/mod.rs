//! Execution Strategy Pattern for Message Processing
//!
//! Two execution strategies:
//! - StandardExecutionStrategy: No tools, pure streaming text generation
//! - PlannedAgenticStrategy: Has tools, PLAN → EXECUTE → RESPOND flow

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use systemprompt_core_ai::{AiMessage, AiService, CallToolResult, ToolCall};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::AgentName;
use systemprompt_models::{ContextId, TaskId};
use tokio::sync::mpsc;

use super::message::StreamEvent;
use crate::models::AgentRuntimeInfo;
use crate::repository::ExecutionStepRepository;
use crate::services::SkillService;

#[derive(Clone)]
pub struct ExecutionContext {
    pub ai_service: Arc<AiService>,
    pub skill_service: Arc<SkillService>,
    pub agent_runtime: AgentRuntimeInfo,
    pub agent_name: AgentName,
    pub task_id: TaskId,
    pub context_id: ContextId,
    pub tx: mpsc::UnboundedSender<StreamEvent>,
    pub log: LogService,
    pub request_ctx: RequestContext,
    pub execution_step_repo: Arc<ExecutionStepRepository>,
}

impl std::fmt::Debug for ExecutionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionContext")
            .field("agent_name", &self.agent_name)
            .field("task_id", &self.task_id)
            .field("context_id", &self.context_id)
            .finish()
    }
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub accumulated_text: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<CallToolResult>,
    pub iterations: usize,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            accumulated_text: String::new(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            iterations: 1,
        }
    }
}

#[async_trait]
pub trait ExecutionStrategy: Send + Sync {
    async fn execute(
        &self,
        context: ExecutionContext,
        messages: Vec<AiMessage>,
    ) -> Result<ExecutionResult>;

    fn name(&self) -> &'static str;
}

pub mod plan_executor;
pub mod planned;
pub mod selector;
pub mod standard;

pub use plan_executor::{
    convert_to_call_tool_results, convert_to_tool_calls, execute_tools_sequentially,
    execute_tools_with_templates, format_results_for_response, ToolExecutorTrait,
};
pub use planned::PlannedAgenticStrategy;
pub use selector::ExecutionStrategySelector;
pub use standard::StandardExecutionStrategy;
