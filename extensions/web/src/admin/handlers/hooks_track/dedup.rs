use sha2::{Digest, Sha256};
use systemprompt::identifiers::{SessionId, UserId};

use crate::admin::types::webhook::{HookEvent, HookEventPayload};

pub fn compute_dedup_key(
    user_id: &UserId,
    session_id: &SessionId,
    payload: &HookEventPayload,
) -> String {
    let raw = build_raw_key(user_id, session_id, payload);
    let hash = Sha256::digest(raw.as_bytes());
    hex::encode(&hash[..16])
}

fn build_raw_key(user_id: &UserId, session_id: &SessionId, payload: &HookEventPayload) -> String {
    let session = session_id.as_str();
    let uid = user_id.as_str();

    match &payload.event {
        HookEvent::PreToolUse(_) => {
            unreachable!("PreToolUse events are dropped before this point")
        }
        HookEvent::PostToolUse(d) => dedup_post_tool_use(uid, session, d),
        HookEvent::PostToolUseFailure(d) => dedup_post_tool_failure(uid, session, d),
        HookEvent::PermissionRequest(d) => {
            let ts = chrono::Utc::now().timestamp();
            format!("{uid}:{session}:PermissionRequest:{}:{ts}", d.tool_name)
        }
        HookEvent::UserPromptSubmit(d) => {
            let h = Sha256::digest(d.prompt.as_bytes());
            let prompt_hash = hex::encode(&h[..8]);
            format!("{uid}:{session}:UserPromptSubmit:{prompt_hash}")
        }
        HookEvent::SessionStart(_) | HookEvent::SessionEnd(_) => {
            format!("{uid}:{session}:{}", payload.event_name())
        }
        HookEvent::TaskCompleted(d) => {
            format!("{uid}:{session}:TaskCompleted:{}", d.task_id)
        }
        HookEvent::SubagentStop(_) => {
            let agent_id = payload.common.agent_id.as_deref().unwrap_or("");
            format!("{uid}:{session}:SubagentStop:{agent_id}")
        }
        HookEvent::SubagentStart(_) => {
            let agent_id = payload.common.agent_id.as_deref().unwrap_or("");
            format!("{uid}:{session}:SubagentStart:{agent_id}")
        }
        HookEvent::Stop(_) => {
            let ts = chrono::Utc::now().timestamp();
            format!("{uid}:{session}:Stop:{ts}")
        }
        HookEvent::TeammateIdle(d) => {
            let ts = chrono::Utc::now().timestamp();
            format!("{uid}:{session}:TeammateIdle:{}:{ts}", d.teammate_name)
        }
        HookEvent::Notification(_)
        | HookEvent::ConfigChange(_)
        | HookEvent::WorktreeCreate(_)
        | HookEvent::WorktreeRemove(_)
        | HookEvent::PreCompact(_)
        | HookEvent::InstructionsLoaded(_)
        | HookEvent::Unknown(_) => uuid::Uuid::new_v4().to_string(),
    }
}

fn dedup_post_tool_use(
    uid: &str,
    session: &str,
    d: &crate::admin::types::webhook::PostToolUseData,
) -> String {
    if d.tool_use_id.is_empty() {
        let ts = chrono::Utc::now().timestamp();
        format!("{uid}:{session}:PostToolUse:{}:{ts}", d.tool_name)
    } else {
        format!("{uid}:{session}:PostToolUse:{}", d.tool_use_id)
    }
}

fn dedup_post_tool_failure(
    uid: &str,
    session: &str,
    d: &crate::admin::types::webhook::PostToolUseFailureData,
) -> String {
    if d.tool_use_id.is_empty() {
        let ts = chrono::Utc::now().timestamp();
        format!("{uid}:{session}:PostToolUseFailure:{}:{ts}", d.tool_name)
    } else {
        format!("{uid}:{session}:PostToolUseFailure:{}", d.tool_use_id)
    }
}
