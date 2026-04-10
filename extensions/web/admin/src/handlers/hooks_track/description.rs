use crate::types::webhook::{HookEvent, HookEventPayload};

use super::helpers::{truncate, value_field};

pub fn generate_description(payload: &HookEventPayload) -> String {
    match &payload.event {
        HookEvent::UserPromptSubmit(d) => describe_user_prompt(d),
        HookEvent::PreToolUse(_) => unreachable!("PreToolUse events are dropped before this point"),
        HookEvent::PostToolUseFailure(d) => describe_post_tool_failure(d),
        HookEvent::PostToolUse(d) => describe_post_tool_use(d),
        HookEvent::PermissionRequest(d) => describe_permission_request(d),
        HookEvent::SessionStart(d) => describe_session_start(d),
        HookEvent::SessionEnd(d) => describe_session_end(d),
        HookEvent::Stop(d) => describe_stop(d),
        HookEvent::SubagentStart(_) => describe_subagent_start(payload),
        HookEvent::SubagentStop(d) => describe_subagent_stop(d),
        HookEvent::TaskCompleted(d) => describe_task_completed(d),
        HookEvent::TeammateIdle(d) => describe_teammate_idle(d),
        HookEvent::Notification(d) => describe_notification(d),
        HookEvent::ConfigChange(d) => describe_config_change(d),
        HookEvent::WorktreeCreate(d) => describe_worktree_create(d),
        HookEvent::WorktreeRemove(d) => describe_worktree_remove(d),
        HookEvent::PreCompact(d) => describe_pre_compact(d),
        HookEvent::InstructionsLoaded(d) => describe_instructions_loaded(d),
        HookEvent::Unknown(name) => name.clone(),
    }
}

fn describe_user_prompt(d: &crate::types::webhook::UserPromptSubmitData) -> String {
    if d.prompt.is_empty() {
        "User prompt submitted".to_string()
    } else {
        format!("Asked: {}", truncate(&d.prompt, 120))
    }
}

fn describe_post_tool_failure(d: &crate::types::webhook::PostToolUseFailureData) -> String {
    let tool = if d.tool_name.is_empty() {
        "unknown"
    } else {
        &d.tool_name
    };
    if d.error.is_empty() {
        format!("Failed: {tool}")
    } else {
        format!("Failed: {tool} — {}", truncate(&d.error, 80))
    }
}

fn describe_post_tool_use(d: &crate::types::webhook::PostToolUseData) -> String {
    let tool = if d.tool_name.is_empty() {
        "unknown"
    } else {
        &d.tool_name
    };
    let detail = value_field(&d.tool_input, "file_path")
        .or_else(|| value_field(&d.tool_input, "command").map(|c| truncate(&c, 60)))
        .or_else(|| value_field(&d.tool_input, "pattern"));
    detail.map_or_else(
        || format!("Completed: {tool}"),
        |det| format!("Completed: {tool} — {det}"),
    )
}

fn describe_permission_request(d: &crate::types::webhook::PermissionRequestData) -> String {
    let tool = if d.tool_name.is_empty() {
        "unknown"
    } else {
        &d.tool_name
    };
    format!("Permission requested: {tool}")
}

fn describe_session_start(d: &crate::types::webhook::SessionStartData) -> String {
    let model = if d.model.is_empty() {
        "unknown"
    } else {
        &d.model
    };
    format!("Session started ({model})")
}

fn describe_session_end(d: &crate::types::webhook::SessionEndData) -> String {
    let reason = if d.reason.is_empty() {
        "completed"
    } else {
        &d.reason
    };
    format!("Session ended ({reason})")
}

fn describe_stop(d: &crate::types::webhook::StopData) -> String {
    d.last_assistant_message.as_ref().map_or_else(
        || "Turn completed".to_string(),
        |msg| format!("AI: {}", truncate(msg, 120)),
    )
}

fn describe_subagent_start(payload: &HookEventPayload) -> String {
    let agent_type = payload.common.agent_type.as_deref().unwrap_or("unknown");
    format!("Subagent started ({agent_type})")
}

fn describe_subagent_stop(d: &crate::types::webhook::SubagentStopData) -> String {
    d.last_assistant_message.as_ref().map_or_else(
        || "Subagent finished".to_string(),
        |msg| format!("Subagent: {}", truncate(msg, 120)),
    )
}

fn describe_task_completed(d: &crate::types::webhook::TaskCompletedData) -> String {
    let subject = if d.task_subject.is_empty() {
        "untitled task"
    } else {
        &d.task_subject
    };
    format!("Task done: {}", truncate(subject, 80))
}

fn describe_teammate_idle(d: &crate::types::webhook::TeammateIdleData) -> String {
    format!("Teammate idle: {}", d.teammate_name)
}

fn describe_notification(d: &crate::types::webhook::NotificationData) -> String {
    let ntype = if d.notification_type.is_empty() {
        "notification"
    } else {
        &d.notification_type
    };
    if d.message.is_empty() {
        format!("Notification ({ntype})")
    } else {
        format!("Notification ({ntype}): {}", truncate(&d.message, 80))
    }
}

fn describe_config_change(d: &crate::types::webhook::ConfigChangeData) -> String {
    let source = if d.source.is_empty() {
        "unknown"
    } else {
        &d.source
    };
    d.file_path.as_ref().map_or_else(
        || format!("Config changed ({source})"),
        |path| format!("Config changed ({source}): {path}"),
    )
}

fn describe_worktree_create(d: &crate::types::webhook::WorktreeCreateData) -> String {
    format!("Worktree created: {}", d.name)
}

fn describe_worktree_remove(d: &crate::types::webhook::WorktreeRemoveData) -> String {
    format!("Worktree removed: {}", d.worktree_path)
}

fn describe_pre_compact(d: &crate::types::webhook::PreCompactData) -> String {
    let trigger = if d.trigger.is_empty() {
        "unknown"
    } else {
        &d.trigger
    };
    format!("Context compaction ({trigger})")
}

fn describe_instructions_loaded(d: &crate::types::webhook::InstructionsLoadedData) -> String {
    if d.load_reason.is_empty() {
        format!("Instructions loaded: {}", d.file_path)
    } else {
        format!("Instructions loaded ({}): {}", d.load_reason, d.file_path)
    }
}
