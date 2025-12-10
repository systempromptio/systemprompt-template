use crate::models::tools::{CallToolResult, ToolCall};
use crate::services::tooled::ToolResultFormatter;

#[derive(Debug)]
pub enum FallbackReason {
    EmptyContent,
    SynthesisFailed(String),
}

#[derive(Debug, Copy, Clone)]
pub struct FallbackGenerator;

impl FallbackGenerator {
    pub const fn new() -> Self {
        Self
    }

    pub fn generate(
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        reason: FallbackReason,
    ) -> String {
        let summary = ToolResultFormatter::format_fallback_summary(tool_calls, tool_results);

        match reason {
            FallbackReason::EmptyContent => {
                format!("Tool execution completed:\n\n{summary}")
            },
            FallbackReason::SynthesisFailed(error) => {
                format!("Tool execution completed:\n\n{summary}\n\n(Synthesis error: {error})")
            },
        }
    }
}

impl Default for FallbackGenerator {
    fn default() -> Self {
        Self::new()
    }
}
