use std::borrow::Cow;

use crate::types::webhook::{HookEvent, HookEventPayload};

use super::helpers::{truncate, value_field};

pub fn generate_description(payload: &HookEventPayload) -> Cow<'static, str> {
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
        HookEvent::Unknown(name) => Cow::Owned(name.clone()),
    }
}

fn describe_user_prompt(d: &crate::types::webhook::UserPromptSubmitData) -> Cow<'static, str> {
    if d.prompt.is_empty() {
        Cow::Borrowed("User prompt submitted")
    } else {
        Cow::Owned(format!("Asked: {}", truncate(&d.prompt, 120)))
    }
}

fn describe_post_tool_failure(d: &crate::types::webhook::PostToolUseFailureData) -> Cow<'static, str> {
    let tool = if d.tool_name.is_empty() {
        "unknown"
    } else {
        &d.tool_name
    };
    if d.error.is_empty() {
        Cow::Owned(format!("Failed: {tool}"))
    } else {
        Cow::Owned(format!("Failed: {tool} — {}", truncate(&d.error, 80)))
    }
}

fn describe_post_tool_use(d: &crate::types::webhook::PostToolUseData) -> Cow<'static, str> {
    let tool = if d.tool_name.is_empty() {
        "unknown"
    } else {
        &d.tool_name
    };
    let detail = value_field(&d.tool_input, "file_path")
        .or_else(|| value_field(&d.tool_input, "command").map(|c| truncate(&c, 60)))
        .or_else(|| value_field(&d.tool_input, "pattern"));
    Cow::Owned(detail.map_or_else(
        || format!("Completed: {tool}"),
        |det| format!("Completed: {tool} — {det}"),
    ))
}

fn describe_permission_request(d: &crate::types::webhook::PermissionRequestData) -> Cow<'static, str> {
    let tool = if d.tool_name.is_empty() {
        "unknown"
    } else {
        &d.tool_name
    };
    Cow::Owned(format!("Permission requested: {tool}"))
}

fn describe_session_start(d: &crate::types::webhook::SessionStartData) -> Cow<'static, str> {
    let model = if d.model.is_empty() {
        "unknown"
    } else {
        &d.model
    };
    Cow::Owned(format!("Session started ({model})"))
}

fn describe_session_end(d: &crate::types::webhook::SessionEndData) -> Cow<'static, str> {
    let reason = if d.reason.is_empty() {
        "completed"
    } else {
        &d.reason
    };
    Cow::Owned(format!("Session ended ({reason})"))
}

fn describe_stop(d: &crate::types::webhook::StopData) -> Cow<'static, str> {
    d.last_assistant_message.as_ref().map_or_else(
        || Cow::Borrowed("Turn completed"),
        |msg| Cow::Owned(format!("AI: {}", truncate(msg, 120))),
    )
}

fn describe_subagent_start(payload: &HookEventPayload) -> Cow<'static, str> {
    let agent_type = payload.common.agent_type.as_deref().unwrap_or("unknown");
    Cow::Owned(format!("Subagent started ({agent_type})"))
}

fn describe_subagent_stop(d: &crate::types::webhook::SubagentStopData) -> Cow<'static, str> {
    d.last_assistant_message.as_ref().map_or_else(
        || Cow::Borrowed("Subagent finished"),
        |msg| Cow::Owned(format!("Subagent: {}", truncate(msg, 120))),
    )
}

fn describe_task_completed(d: &crate::types::webhook::TaskCompletedData) -> Cow<'static, str> {
    let subject = if d.task_subject.is_empty() {
        "untitled task"
    } else {
        &d.task_subject
    };
    Cow::Owned(format!("Task done: {}", truncate(subject, 80)))
}

fn describe_teammate_idle(d: &crate::types::webhook::TeammateIdleData) -> Cow<'static, str> {
    Cow::Owned(format!("Teammate idle: {}", d.teammate_name))
}

fn describe_notification(d: &crate::types::webhook::NotificationData) -> Cow<'static, str> {
    let ntype = if d.notification_type.is_empty() {
        "notification"
    } else {
        &d.notification_type
    };
    if d.message.is_empty() {
        Cow::Owned(format!("Notification ({ntype})"))
    } else {
        Cow::Owned(format!("Notification ({ntype}): {}", truncate(&d.message, 80)))
    }
}

fn describe_config_change(d: &crate::types::webhook::ConfigChangeData) -> Cow<'static, str> {
    let source = if d.source.is_empty() {
        "unknown"
    } else {
        &d.source
    };
    Cow::Owned(d.file_path.as_ref().map_or_else(
        || format!("Config changed ({source})"),
        |path| format!("Config changed ({source}): {path}"),
    ))
}

fn describe_worktree_create(d: &crate::types::webhook::WorktreeCreateData) -> Cow<'static, str> {
    Cow::Owned(format!("Worktree created: {}", d.name))
}

fn describe_worktree_remove(d: &crate::types::webhook::WorktreeRemoveData) -> Cow<'static, str> {
    Cow::Owned(format!("Worktree removed: {}", d.worktree_path))
}

fn describe_pre_compact(d: &crate::types::webhook::PreCompactData) -> Cow<'static, str> {
    let trigger = if d.trigger.is_empty() {
        "unknown"
    } else {
        &d.trigger
    };
    Cow::Owned(format!("Context compaction ({trigger})"))
}

fn describe_instructions_loaded(d: &crate::types::webhook::InstructionsLoadedData) -> Cow<'static, str> {
    if d.load_reason.is_empty() {
        Cow::Owned(format!("Instructions loaded: {}", d.file_path))
    } else {
        Cow::Owned(format!("Instructions loaded ({}): {}", d.load_reason, d.file_path))
    }
}
