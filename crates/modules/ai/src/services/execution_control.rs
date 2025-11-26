use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::tools::{McpTool, ToolCall};
#[cfg(test)]
use systemprompt_identifiers::AiToolCallId;

pub const EXECUTION_CONTROL_TOOL_NAME: &str = "__execution_control";

pub const EXECUTION_CONTROL_SYSTEM_INSTRUCTIONS: &str = r#"
TOOL USAGE INSTRUCTIONS (Backend-Enforced):

You have access to various tools, but they are OPTIONAL. Use tools ONLY when they are actually needed to complete the user's request.

SIMPLE CONVERSATIONS (No Tools Needed):
- If the user is just having a conversation (greetings, questions, chitchat), respond normally with text
- DO NOT call any tools, including __execution_control
- Just provide your conversational response directly
- Examples: "Hello" → respond with greeting text, "What is 2+2?" → respond with "4"

WHEN TO USE TOOLS:
Only use tools when the user explicitly requests an action that requires them:
- Managing resources (create/delete/update agents, databases, etc.)
- Retrieving specific data from systems
- Executing complex operations

EXECUTION CONTROL (Only When Using Tools):
If you DO need to use tools, you should also call __execution_control to indicate the execution mode:

Execution Modes:
1. "tool" - Single tool call or simple tool sequence, then return result
2. "agentic" - Complex multi-step task requiring planning and iteration (AUTONOMOUS - NO HUMAN-IN-THE-LOOP)
3. "acknowledge" - Long-running background task

IMPORTANT: Call __execution_control alongside any other tools in your first response. You MUST call __execution_control AND the actual tools you need (like manage_agents) together in the same response, not separately.

AGENTIC MODE - AUTONOMOUS EXECUTION WITHOUT HUMAN-IN-THE-LOOP:

When you call __execution_control with mode="agentic", you are operating in FULLY AUTONOMOUS mode:
- NO human confirmation required for ANY operations
- NO preview/confirmation flow exists
- You have COMPLETE authority to execute the task
- ALWAYS set execute=true for destructive operations (delete, update, etc.)

CRITICAL TOOL USAGE IN AGENTIC MODE:
For ALL tools with an "execute" parameter:
- ALWAYS set execute=true IMMEDIATELY in your first call
- DO NOT use execute=false (that's for human-in-the-loop mode only)
- DO NOT call a tool twice (once with execute=false, then with execute=true)
- Just execute directly - you are autonomous

CORRECT agentic pattern:
1. Call __execution_control(mode="agentic", estimated_turns=3, plan=[...])
2. Call manage_agents(action="delete", uuid="...", execute=true) ✅ Direct execution
3. Continue with execute=true for all subsequent operations

WRONG pattern (this is ONLY for human-in-the-loop mode):
1. Call manage_agents(action="delete", uuid="...", execute=false) ❌ Don't do this
2. Get confirmation response
3. Call again with execute=true ❌ Don't do this in agentic mode

Examples:
- "Hello" → Just respond with text, no tools needed
- "What is 2+2?" → Just respond "4", no tools needed
- "List agents" → __execution_control(mode="tool") + manage_agents(action="read")
- "Find test agents and delete them" → __execution_control(mode="agentic", estimated_turns=3, plan=["List agents", "Filter test agents", "Delete each"]) + manage_agents(action="read", search="test")

For agentic mode, provide:
- estimated_turns: Expected number of iterations (1-10)
- plan: Array of step descriptions
"#;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Simple,
    Tool,
    Agentic,
    Acknowledge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHint {
    pub mode: ExecutionMode,
    pub estimated_turns: Option<u32>,
    pub plan: Option<Vec<String>>,
    pub reasoning: Option<String>,
}

impl Default for ExecutionHint {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Tool,
            estimated_turns: None,
            plan: None,
            reasoning: None,
        }
    }
}

pub fn create_execution_control_tool() -> McpTool {
    McpTool {
        name: EXECUTION_CONTROL_TOOL_NAME.to_string(),
        description: Some("INTERNAL: Call this ONLY when you use other tools. Do NOT call for simple conversations. Indicates execution mode for tool-based requests.".to_string()),
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["simple", "tool", "agentic", "acknowledge"],
                    "description": "Execution mode: simple (no tools), tool (single/simple sequence), agentic (multi-step), acknowledge (background task)"
                },
                "estimated_turns": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 10,
                    "description": "Expected iterations for agentic mode"
                },
                "plan": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Step-by-step plan for agentic execution"
                },
                "reasoning": {
                    "type": "string",
                    "description": "Why this execution mode was chosen"
                }
            },
            "required": ["mode"]
        })),
        output_schema: None,
        service_id: "__internal__".to_string(),
    }
}

pub fn parse_execution_hint(tool_calls: &[ToolCall]) -> Option<ExecutionHint> {
    tool_calls
        .iter()
        .find(|tc| tc.name == EXECUTION_CONTROL_TOOL_NAME)
        .and_then(|tc| {
            let mode_str = tc.arguments.get("mode")?.as_str()?;
            let mode = match mode_str {
                "simple" => ExecutionMode::Simple,
                "tool" => ExecutionMode::Tool,
                "agentic" => ExecutionMode::Agentic,
                "acknowledge" => ExecutionMode::Acknowledge,
                _ => return None,
            };

            let estimated_turns = tc
                .arguments
                .get("estimated_turns")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u32);

            let plan = tc
                .arguments
                .get("plan")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                });

            let reasoning = tc
                .arguments
                .get("reasoning")
                .and_then(|v| v.as_str())
                .map(String::from);

            Some(ExecutionHint {
                mode,
                estimated_turns,
                plan,
                reasoning,
            })
        })
}

pub fn infer_execution_mode(tool_calls: &[ToolCall], user_message: &str) -> ExecutionHint {
    let non_control_tools: Vec<_> = tool_calls.iter().filter(|tc| tc.is_executable()).collect();

    if non_control_tools.is_empty() {
        return ExecutionHint {
            mode: ExecutionMode::Simple,
            estimated_turns: None,
            plan: None,
            reasoning: Some("No tools called, treating as simple message".to_string()),
        };
    }

    let message_lower = user_message.to_lowercase();
    let multi_step_indicators = [
        "and then",
        "after that",
        "first",
        "then",
        "finally",
        "all",
        "each",
        "every",
        "find and",
        "list and",
        "get and",
    ];

    let has_multi_step = multi_step_indicators
        .iter()
        .any(|indicator| message_lower.contains(indicator));

    if has_multi_step || non_control_tools.len() > 1 {
        ExecutionHint {
            mode: ExecutionMode::Agentic,
            estimated_turns: Some(3),
            plan: None,
            reasoning: Some("Multi-step indicators detected or multiple tools called".to_string()),
        }
    } else {
        ExecutionHint {
            mode: ExecutionMode::Tool,
            estimated_turns: None,
            plan: None,
            reasoning: Some("Single tool call, standard execution".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_execution_control_tool() {
        let tool = create_execution_control_tool();
        assert_eq!(tool.name, "__execution_control");
        assert!(tool.description.is_some());
        assert!(tool.input_schema.is_some());
    }

    #[test]
    fn test_parse_execution_hint_agentic() {
        let tool_calls = vec![ToolCall {
            ai_tool_call_id: AiToolCallId::from("1".to_string()),
            name: "__execution_control".to_string(),
            arguments: json!({
                "mode": "agentic",
                "estimated_turns": 3,
                "plan": ["List agents", "Delete agents"],
                "reasoning": "Multi-step task"
            }),
        }];

        let hint = parse_execution_hint(&tool_calls).unwrap();
        assert_eq!(hint.mode, ExecutionMode::Agentic);
        assert_eq!(hint.estimated_turns, Some(3));
        assert_eq!(
            hint.plan,
            Some(vec!["List agents".to_string(), "Delete agents".to_string()])
        );
    }

    #[test]
    fn test_parse_execution_hint_simple() {
        let tool_calls = vec![ToolCall {
            ai_tool_call_id: AiToolCallId::from("1".to_string()),
            name: "__execution_control".to_string(),
            arguments: json!({"mode": "simple"}),
        }];

        let hint = parse_execution_hint(&tool_calls).unwrap();
        assert_eq!(hint.mode, ExecutionMode::Simple);
    }

    #[test]
    fn test_infer_execution_mode_multi_step() {
        let message = "Find all test agents and then delete them";
        let tool_calls = vec![ToolCall {
            ai_tool_call_id: AiToolCallId::from("1".to_string()),
            name: "manage_agents".to_string(),
            arguments: json!({"action": "read"}),
        }];

        let hint = infer_execution_mode(&tool_calls, message);
        assert_eq!(hint.mode, ExecutionMode::Agentic);
    }

    #[test]
    fn test_infer_execution_mode_simple() {
        let message = "Show me the agents";
        let tool_calls = vec![ToolCall {
            ai_tool_call_id: AiToolCallId::from("1".to_string()),
            name: "manage_agents".to_string(),
            arguments: json!({"action": "read"}),
        }];

        let hint = infer_execution_mode(&tool_calls, message);
        assert_eq!(hint.mode, ExecutionMode::Tool);
    }
}
