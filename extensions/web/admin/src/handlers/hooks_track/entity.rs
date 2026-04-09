use crate::types::webhook::HookEventPayload;

pub fn detect_entity(payload: &HookEventPayload) -> Option<(&'static str, String)> {
    let tool_name = payload.tool_name();

    if tool_name == Some("Skill") {
        return detect_skill_entity(payload);
    }

    if let Some(tn) = tool_name {
        if let Some(server) = detect_mcp_server(tn) {
            return Some(("mcp_tool", server));
        }
    }

    if tool_name == Some("Agent") {
        return detect_agent_entity(payload);
    }

    if matches!(payload.event_name(), "SubagentStart" | "SubagentStop") {
        let agent_type = payload
            .common
            .agent_type
            .clone()
            .unwrap_or_else(|| "subagent".to_string());
        return Some(("agent", agent_type));
    }

    None
}

fn detect_skill_entity(payload: &HookEventPayload) -> Option<(&'static str, String)> {
    let input = payload.tool_input()?;
    let skill = input.get("skill")?.as_str()?;
    (!skill.is_empty()).then(|| ("skill", skill.to_string()))
}

fn detect_mcp_server(tool_name: &str) -> Option<String> {
    let server = tool_name
        .strip_prefix("mcp__")
        .and_then(|rest| rest.split("__").next())?;
    if server.is_empty() {
        None
    } else {
        Some(server.to_string())
    }
}

fn detect_agent_entity(payload: &HookEventPayload) -> Option<(&'static str, String)> {
    let input = payload.tool_input()?;
    let name = input
        .get("subagent_type")
        .and_then(|v| v.as_str())
        .or_else(|| input.get("description").and_then(|v| v.as_str()))
        .unwrap_or("subagent");
    Some(("agent", name.to_string()))
}
