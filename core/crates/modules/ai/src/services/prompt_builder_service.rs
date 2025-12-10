use std::sync::Arc;

use crate::{AiMessage, MessageRole};

#[derive(Debug, Clone, Copy)]
pub struct PromptBuilderService;

impl PromptBuilderService {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    pub fn build_prompt_with_system(
        system_prompt: Option<&str>,
        conversation_history: Vec<AiMessage>,
    ) -> Vec<AiMessage> {
        let mut messages = Vec::new();

        if let Some(prompt) = system_prompt {
            messages.push(AiMessage {
                role: MessageRole::System,
                content: prompt.to_string(),
            });
        }

        messages.extend(conversation_history);
        messages
    }

    pub fn build_prompt_with_system_and_user_message(
        system_prompt: Option<&str>,
        conversation_history: Vec<AiMessage>,
        user_message: &str,
    ) -> Vec<AiMessage> {
        let mut messages = Self::build_prompt_with_system(system_prompt, conversation_history);

        messages.push(AiMessage {
            role: MessageRole::User,
            content: user_message.to_string(),
        });

        messages
    }

    pub fn append_synthesis_instruction(
        messages: &mut Vec<AiMessage>,
        current_step: i32,
        original_request: &str,
        tool_results_summary: &str,
    ) {
        let instruction = format!(
            r#"You are analyzing the results of tool execution to decide the next action.

ORIGINAL USER REQUEST:
{original_request}

CURRENT STEP: {current_step}

TOOL EXECUTION RESULTS:
{tool_results_summary}

Based on the tool results and the original request, provide a structured decision with these fields:

1. synthesized_response: Clear message for the user explaining what has been done
2. decision: "return" if task is complete, or "continue" if more work needed
3. next_tool (if continuing): Name of the next tool to use
4. next_tool_args (if continuing): Arguments for that tool (as JSON object)
5. reasoning: Explanation of your decision-making process
6. current_step: Current step number
7. estimated_total_steps: Estimated total steps to complete the task
8. progress_percentage: Progress percentage (0-100)
9. step_summary: Brief description of what was accomplished

Be precise and actionable. If continuing, ensure next tool and arguments are valid."#
        );

        messages.push(AiMessage {
            role: MessageRole::User,
            content: instruction,
        });
    }
}
