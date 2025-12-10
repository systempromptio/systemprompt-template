use serde_json::Value;

use crate::models::tools::McpTool;

pub fn get_tool_output_schemas(
    calls: &[systemprompt_models::ai::PlannedToolCall],
    available_tools: &[McpTool],
) -> Vec<(String, Option<Value>)> {
    calls
        .iter()
        .map(|call| {
            let output_schema = available_tools
                .iter()
                .find(|t| t.name == call.tool_name)
                .and_then(|t| t.output_schema.clone());
            (call.tool_name.clone(), output_schema)
        })
        .collect()
}
