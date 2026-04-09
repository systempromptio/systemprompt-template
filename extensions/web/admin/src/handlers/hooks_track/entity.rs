use crate::types::webhook::HookEventPayload;

#[derive(serde::Deserialize)]
struct SkillToolInput {
    skill: String,
}

#[derive(serde::Deserialize)]
struct AgentToolInput {
    subagent_type: Option<String>,
    description: Option<String>,
}

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
    let parsed: SkillToolInput = serde_json::from_value(input.clone()).ok()?;
    (!parsed.skill.is_empty()).then_some(("skill", parsed.skill))
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
    let parsed: AgentToolInput = serde_json::from_value(input.clone()).ok()?;
    let name = parsed
        .subagent_type
        .or(parsed.description)
        .unwrap_or_else(|| "subagent".to_string());
    Some(("agent", name))
}
