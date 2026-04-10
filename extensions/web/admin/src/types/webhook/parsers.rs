use super::event_types::{
    ConfigChangeData, HookCommonFields, HookEvent, InstructionsLoadedData, NotificationData,
    PermissionRequestData, PostToolUseData, PostToolUseFailureData, PreCompactData, PreToolUseData,
    SessionEndData, SessionStartData, StopData, SubagentStartData, SubagentStopData,
    TaskCompletedData, TeammateIdleData, UserPromptSubmitData, WorktreeCreateData,
    WorktreeRemoveData,
};

fn parse_event<T: serde::de::DeserializeOwned>(
    raw: &serde_json::Value,
    event_name: &str,
    warnings: &mut Vec<String>,
    wrap: fn(T) -> HookEvent,
) -> HookEvent {
    match serde_json::from_value::<T>(raw.clone()) {
        Ok(data) => wrap(data),
        Err(e) => {
            warnings.push(format!("{event_name} parse error: {e}"));
            HookEvent::Unknown(event_name.to_string())
        }
    }
}

pub fn parse_session_start(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    parse_event::<SessionStartData>(raw, "SessionStart", warnings, HookEvent::SessionStart)
}

pub fn parse_session_end(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    parse_event::<SessionEndData>(raw, "SessionEnd", warnings, HookEvent::SessionEnd)
}

pub fn parse_stop(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    parse_event::<StopData>(raw, "Stop", warnings, HookEvent::Stop)
}

pub fn parse_teammate_idle(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    parse_event::<TeammateIdleData>(raw, "TeammateIdle", warnings, HookEvent::TeammateIdle)
}

pub fn parse_config_change(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    parse_event::<ConfigChangeData>(raw, "ConfigChange", warnings, HookEvent::ConfigChange)
}

pub fn parse_pre_compact(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    parse_event::<PreCompactData>(raw, "PreCompact", warnings, HookEvent::PreCompact)
}

pub fn parse_user_prompt_submit(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<UserPromptSubmitData>(raw, "UserPromptSubmit", warnings, HookEvent::UserPromptSubmit);
    if let HookEvent::UserPromptSubmit(ref data) = event {
        if data.prompt.is_empty() {
            warnings.push("UserPromptSubmit missing required field: prompt".to_string());
        }
    }
    event
}

pub fn parse_pre_tool_use(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<PreToolUseData>(raw, "PreToolUse", warnings, HookEvent::PreToolUse);
    if let HookEvent::PreToolUse(ref data) = event {
        if data.tool_name.is_empty() {
            warnings.push("PreToolUse missing required field: tool_name".to_string());
        }
    }
    event
}

pub fn parse_post_tool_use(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<PostToolUseData>(raw, "PostToolUse", warnings, HookEvent::PostToolUse);
    if let HookEvent::PostToolUse(ref data) = event {
        if data.tool_name.is_empty() {
            warnings.push("PostToolUse missing required field: tool_name".to_string());
        }
    }
    event
}

pub fn parse_post_tool_use_failure(
    raw: &serde_json::Value,
    warnings: &mut Vec<String>,
) -> HookEvent {
    let event = parse_event::<PostToolUseFailureData>(raw, "PostToolUseFailure", warnings, HookEvent::PostToolUseFailure);
    if let HookEvent::PostToolUseFailure(ref data) = event {
        if data.tool_name.is_empty() {
            warnings.push("PostToolUseFailure missing required field: tool_name".to_string());
        }
        if data.error.is_empty() {
            warnings.push("PostToolUseFailure missing required field: error".to_string());
        }
    }
    event
}

pub fn parse_permission_request(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<PermissionRequestData>(raw, "PermissionRequest", warnings, HookEvent::PermissionRequest);
    if let HookEvent::PermissionRequest(ref data) = event {
        if data.tool_name.is_empty() {
            warnings.push("PermissionRequest missing required field: tool_name".to_string());
        }
    }
    event
}

pub fn parse_task_completed(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<TaskCompletedData>(raw, "TaskCompleted", warnings, HookEvent::TaskCompleted);
    if let HookEvent::TaskCompleted(ref data) = event {
        if data.task_id.is_empty() {
            warnings.push("TaskCompleted missing required field: task_id".to_string());
        }
    }
    event
}

pub fn parse_notification(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<NotificationData>(raw, "Notification", warnings, HookEvent::Notification);
    if let HookEvent::Notification(ref data) = event {
        if data.message.is_empty() {
            warnings.push("Notification missing required field: message".to_string());
        }
    }
    event
}

pub fn parse_worktree_create(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<WorktreeCreateData>(raw, "WorktreeCreate", warnings, HookEvent::WorktreeCreate);
    if let HookEvent::WorktreeCreate(ref data) = event {
        if data.name.is_empty() {
            warnings.push("WorktreeCreate missing required field: name".to_string());
        }
    }
    event
}

pub fn parse_worktree_remove(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<WorktreeRemoveData>(raw, "WorktreeRemove", warnings, HookEvent::WorktreeRemove);
    if let HookEvent::WorktreeRemove(ref data) = event {
        if data.worktree_path.is_empty() {
            warnings.push("WorktreeRemove missing required field: worktree_path".to_string());
        }
    }
    event
}

pub fn parse_instructions_loaded(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    let event = parse_event::<InstructionsLoadedData>(raw, "InstructionsLoaded", warnings, HookEvent::InstructionsLoaded);
    if let HookEvent::InstructionsLoaded(ref data) = event {
        if data.file_path.is_empty() {
            warnings.push("InstructionsLoaded missing required field: file_path".to_string());
        }
    }
    event
}

pub fn parse_subagent_start(
    raw: &serde_json::Value,
    common: &HookCommonFields,
    warnings: &mut Vec<String>,
) -> HookEvent {
    let event = parse_event::<SubagentStartData>(raw, "SubagentStart", warnings, HookEvent::SubagentStart);
    if matches!(event, HookEvent::SubagentStart(_)) && common.agent_id.is_none() {
        warnings.push("SubagentStart missing expected common field: agent_id".to_string());
    }
    event
}

pub fn parse_subagent_stop(
    raw: &serde_json::Value,
    common: &HookCommonFields,
    warnings: &mut Vec<String>,
) -> HookEvent {
    let event = parse_event::<SubagentStopData>(raw, "SubagentStop", warnings, HookEvent::SubagentStop);
    if matches!(event, HookEvent::SubagentStop(_)) && common.agent_id.is_none() {
        warnings.push("SubagentStop missing expected common field: agent_id".to_string());
    }
    event
}
