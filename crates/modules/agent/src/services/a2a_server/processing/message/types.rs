use crate::models::a2a::Artifact;
use systemprompt_core_ai::{CallToolResult, ToolCall};

#[derive(Debug)]
pub enum StreamEvent {
    Text(String),
    ToolCallStarted(ToolCall),
    ToolResult {
        call_id: String,
        result: CallToolResult,
    },
    ArtifactUpdate {
        artifact: Artifact,
        append: bool,
        last_chunk: bool,
    },
    ExecutionStepUpdate {
        step: crate::models::ExecutionStep,
    },
    Complete {
        full_text: String,
        artifacts: Vec<Artifact>,
    },
    Error(String),
}
