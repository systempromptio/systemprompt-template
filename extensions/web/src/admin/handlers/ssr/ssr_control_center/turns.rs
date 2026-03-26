use std::collections::HashMap;

use crate::admin::types::control_center::ActivityFeedEvent;

use super::types::{ToolError, ToolGroup, Turn};

pub fn build_turns(chronological: &[&&ActivityFeedEvent]) -> Vec<Turn> {
    let mut turns: Vec<Turn> = Vec::new();
    let mut state = FlushTurnState {
        prompt: None,
        prompt_time: None,
        tools: HashMap::new(),
        errors: Vec::new(),
        response: None,
        response_time: None,
    };

    for evt in chronological {
        let et = &evt.event_type;

        if et.contains("UserPromptSubmit") {
            flush_turn(&mut turns, &mut state);
            state.prompt = Some(evt.prompt_preview.as_deref().unwrap_or("").to_string());
            state.prompt_time = Some(evt.created_at.to_rfc3339());
        } else if et.contains("PostToolUseFailure") {
            let tool = evt.tool_name.as_deref().unwrap_or("Unknown");
            state.errors.push(ToolError {
                tool_name: tool.to_string(),
                description: evt
                    .description
                    .as_deref()
                    .unwrap_or("Tool failed")
                    .to_string(),
            });
            *state.tools.entry(tool.to_string()).or_insert(0) += 1;
        } else if et.contains("PostToolUse") {
            let tool = evt.tool_name.as_deref().unwrap_or("Unknown");
            *state.tools.entry(tool.to_string()).or_insert(0) += 1;
        } else if et.contains("Stop") || et.contains("SubagentStop") {
            state.response = Some(evt.prompt_preview.as_deref().unwrap_or("").to_string());
            state.response_time = Some(evt.created_at.to_rfc3339());
        }
    }

    flush_turn(&mut turns, &mut state);

    for turn in &mut turns {
        turn.has_prompt = !turn.prompt_text.is_empty();
        turn.has_response = !turn.response_text.is_empty();
    }

    turns
}

struct FlushTurnState {
    prompt: Option<String>,
    prompt_time: Option<String>,
    tools: HashMap<String, usize>,
    errors: Vec<ToolError>,
    response: Option<String>,
    response_time: Option<String>,
}

fn flush_turn(turns: &mut Vec<Turn>, state: &mut FlushTurnState) {
    if state.prompt.is_some()
        || state.response.is_some()
        || !state.tools.is_empty()
        || !state.errors.is_empty()
    {
        let mut tool_groups: Vec<(String, usize)> = state.tools.drain().collect();
        tool_groups.sort_by(|a, b| b.1.cmp(&a.1));
        let total_tools: usize = tool_groups.iter().map(|(_, c)| c).sum();
        let tool_groups_typed: Vec<ToolGroup> = tool_groups
            .iter()
            .map(|(name, count)| ToolGroup {
                name: name.clone(),
                count: *count,
            })
            .collect();
        let error_list: Vec<ToolError> = std::mem::take(&mut state.errors);

        turns.push(Turn {
            prompt_text: state.prompt.take().unwrap_or_default(),
            prompt_time: state.prompt_time.take().unwrap_or_default(),
            response_text: state.response.take().unwrap_or_default(),
            response_time: state.response_time.take().unwrap_or_default(),
            tool_groups: tool_groups_typed,
            total_tools,
            has_tools: total_tools > 0,
            has_errors: !error_list.is_empty(),
            errors: error_list,
            has_prompt: false,
            has_response: false,
        });
    }
}

pub fn count_prompts(turns: &[Turn]) -> usize {
    turns.iter().filter(|t| t.has_prompt).count()
}

pub fn sum_tools(turns: &[Turn]) -> usize {
    turns.iter().map(|t| t.total_tools).sum()
}

pub fn sum_errors(turns: &[Turn]) -> usize {
    turns.iter().map(|t| t.errors.len()).sum()
}
