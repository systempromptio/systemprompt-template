use crate::models::ai::{AiMessage, MessageRole};
use crate::models::tools::{CallToolResult, ToolCall};
use crate::services::tooled::ToolResultFormatter;

#[derive(Debug, Copy, Clone)]
pub struct SynthesisPromptBuilder;

impl SynthesisPromptBuilder {
    pub fn build_guidance_message(
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
    ) -> AiMessage {
        let tool_summary = ToolResultFormatter::format_for_display(tool_calls, tool_results);

        AiMessage {
            role: MessageRole::User,
            content: format!(
                "The following tools were just executed:\n\n{tool_summary}\n\n\
                Based on these tool execution results, provide a clear, natural language \
                response to the user. Focus on:\n\
                - What the results mean for the user\n\
                - How they answer the user's question\n\
                - Any important insights from the data\n\n\
                Be concise but informative. Do not repeat the raw tool data - \
                synthesize it into a helpful response."
            ),
        }
    }
}
