use super::event_types::{
    ConfigChangeData, HookCommonFields, HookEvent, InstructionsLoadedData, NotificationData,
    PermissionRequestData, PostToolUseData, PostToolUseFailureData, PreCompactData, PreToolUseData,
    SessionEndData, SessionStartData, StopData, SubagentStartData, SubagentStopData,
    TaskCompletedData, TeammateIdleData, UserPromptSubmitData, WorktreeCreateData,
    WorktreeRemoveData,
};

pub fn parse_session_start(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<SessionStartData>(raw.clone()) {
        Ok(data) => HookEvent::SessionStart(data),
        Err(e) => {
            warnings.push(format!("SessionStart parse error: {e}"));
            HookEvent::Unknown("SessionStart".to_string())
        }
    }
}

pub fn parse_session_end(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<SessionEndData>(raw.clone()) {
        Ok(data) => HookEvent::SessionEnd(data),
        Err(e) => {
            warnings.push(format!("SessionEnd parse error: {e}"));
            HookEvent::Unknown("SessionEnd".to_string())
        }
    }
}

pub fn parse_user_prompt_submit(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<UserPromptSubmitData>(raw.clone()) {
        Ok(data) => {
            if data.prompt.is_empty() {
                warnings.push("UserPromptSubmit missing required field: prompt".to_string());
            }
            HookEvent::UserPromptSubmit(data)
        }
        Err(e) => {
            warnings.push(format!("UserPromptSubmit parse error: {e}"));
            HookEvent::Unknown("UserPromptSubmit".to_string())
        }
    }
}

pub fn parse_pre_tool_use(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<PreToolUseData>(raw.clone()) {
        Ok(data) => {
            if data.tool_name.is_empty() {
                warnings.push("PreToolUse missing required field: tool_name".to_string());
            }
            HookEvent::PreToolUse(data)
        }
        Err(e) => {
            warnings.push(format!("PreToolUse parse error: {e}"));
            HookEvent::Unknown("PreToolUse".to_string())
        }
    }
}

pub fn parse_post_tool_use(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<PostToolUseData>(raw.clone()) {
        Ok(data) => {
            if data.tool_name.is_empty() {
                warnings.push("PostToolUse missing required field: tool_name".to_string());
            }
            HookEvent::PostToolUse(data)
        }
        Err(e) => {
            warnings.push(format!("PostToolUse parse error: {e}"));
            HookEvent::Unknown("PostToolUse".to_string())
        }
    }
}

pub fn parse_post_tool_use_failure(
    raw: &serde_json::Value,
    warnings: &mut Vec<String>,
) -> HookEvent {
    match serde_json::from_value::<PostToolUseFailureData>(raw.clone()) {
        Ok(data) => {
            if data.tool_name.is_empty() {
                warnings.push("PostToolUseFailure missing required field: tool_name".to_string());
            }
            if data.error.is_empty() {
                warnings.push("PostToolUseFailure missing required field: error".to_string());
            }
            HookEvent::PostToolUseFailure(data)
        }
        Err(e) => {
            warnings.push(format!("PostToolUseFailure parse error: {e}"));
            HookEvent::Unknown("PostToolUseFailure".to_string())
        }
    }
}

pub fn parse_permission_request(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<PermissionRequestData>(raw.clone()) {
        Ok(data) => {
            if data.tool_name.is_empty() {
                warnings.push("PermissionRequest missing required field: tool_name".to_string());
            }
            HookEvent::PermissionRequest(data)
        }
        Err(e) => {
            warnings.push(format!("PermissionRequest parse error: {e}"));
            HookEvent::Unknown("PermissionRequest".to_string())
        }
    }
}

pub fn parse_stop(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<StopData>(raw.clone()) {
        Ok(data) => HookEvent::Stop(data),
        Err(e) => {
            warnings.push(format!("Stop parse error: {e}"));
            HookEvent::Unknown("Stop".to_string())
        }
    }
}

pub fn parse_subagent_start(
    raw: &serde_json::Value,
    common: &HookCommonFields,
    warnings: &mut Vec<String>,
) -> HookEvent {
    match serde_json::from_value::<SubagentStartData>(raw.clone()) {
        Ok(data) => {
            if common.agent_id.is_none() {
                warnings.push("SubagentStart missing expected common field: agent_id".to_string());
            }
            HookEvent::SubagentStart(data)
        }
        Err(e) => {
            warnings.push(format!("SubagentStart parse error: {e}"));
            HookEvent::Unknown("SubagentStart".to_string())
        }
    }
}

pub fn parse_subagent_stop(
    raw: &serde_json::Value,
    common: &HookCommonFields,
    warnings: &mut Vec<String>,
) -> HookEvent {
    match serde_json::from_value::<SubagentStopData>(raw.clone()) {
        Ok(data) => {
            if common.agent_id.is_none() {
                warnings.push("SubagentStop missing expected common field: agent_id".to_string());
            }
            HookEvent::SubagentStop(data)
        }
        Err(e) => {
            warnings.push(format!("SubagentStop parse error: {e}"));
            HookEvent::Unknown("SubagentStop".to_string())
        }
    }
}

pub fn parse_task_completed(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<TaskCompletedData>(raw.clone()) {
        Ok(data) => {
            if data.task_id.is_empty() {
                warnings.push("TaskCompleted missing required field: task_id".to_string());
            }
            HookEvent::TaskCompleted(data)
        }
        Err(e) => {
            warnings.push(format!("TaskCompleted parse error: {e}"));
            HookEvent::Unknown("TaskCompleted".to_string())
        }
    }
}

pub fn parse_teammate_idle(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<TeammateIdleData>(raw.clone()) {
        Ok(data) => HookEvent::TeammateIdle(data),
        Err(e) => {
            warnings.push(format!("TeammateIdle parse error: {e}"));
            HookEvent::Unknown("TeammateIdle".to_string())
        }
    }
}

pub fn parse_notification(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<NotificationData>(raw.clone()) {
        Ok(data) => {
            if data.message.is_empty() {
                warnings.push("Notification missing required field: message".to_string());
            }
            HookEvent::Notification(data)
        }
        Err(e) => {
            warnings.push(format!("Notification parse error: {e}"));
            HookEvent::Unknown("Notification".to_string())
        }
    }
}

pub fn parse_config_change(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<ConfigChangeData>(raw.clone()) {
        Ok(data) => HookEvent::ConfigChange(data),
        Err(e) => {
            warnings.push(format!("ConfigChange parse error: {e}"));
            HookEvent::Unknown("ConfigChange".to_string())
        }
    }
}

pub fn parse_worktree_create(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<WorktreeCreateData>(raw.clone()) {
        Ok(data) => {
            if data.name.is_empty() {
                warnings.push("WorktreeCreate missing required field: name".to_string());
            }
            HookEvent::WorktreeCreate(data)
        }
        Err(e) => {
            warnings.push(format!("WorktreeCreate parse error: {e}"));
            HookEvent::Unknown("WorktreeCreate".to_string())
        }
    }
}

pub fn parse_worktree_remove(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<WorktreeRemoveData>(raw.clone()) {
        Ok(data) => {
            if data.worktree_path.is_empty() {
                warnings.push("WorktreeRemove missing required field: worktree_path".to_string());
            }
            HookEvent::WorktreeRemove(data)
        }
        Err(e) => {
            warnings.push(format!("WorktreeRemove parse error: {e}"));
            HookEvent::Unknown("WorktreeRemove".to_string())
        }
    }
}

pub fn parse_pre_compact(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<PreCompactData>(raw.clone()) {
        Ok(data) => HookEvent::PreCompact(data),
        Err(e) => {
            warnings.push(format!("PreCompact parse error: {e}"));
            HookEvent::Unknown("PreCompact".to_string())
        }
    }
}

pub fn parse_instructions_loaded(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookEvent {
    match serde_json::from_value::<InstructionsLoadedData>(raw.clone()) {
        Ok(data) => {
            if data.file_path.is_empty() {
                warnings.push("InstructionsLoaded missing required field: file_path".to_string());
            }
            HookEvent::InstructionsLoaded(data)
        }
        Err(e) => {
            warnings.push(format!("InstructionsLoaded parse error: {e}"));
            HookEvent::Unknown("InstructionsLoaded".to_string())
        }
    }
}
