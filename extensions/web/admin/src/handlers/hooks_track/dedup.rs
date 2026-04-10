use std::fmt::Write;

use sha2::{Digest, Sha256};
use systemprompt::identifiers::{SessionId, UserId};

use crate::types::webhook::{HookEvent, HookEventPayload};

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
    let mut key = String::with_capacity(128);

    match &payload.event {
        HookEvent::PreToolUse(_) => {
            unreachable!("PreToolUse events are dropped before this point")
        }
        HookEvent::PostToolUse(d) => write_post_tool_use(&mut key, uid, session, d),
        HookEvent::PostToolUseFailure(d) => write_post_tool_failure(&mut key, uid, session, d),
        HookEvent::PermissionRequest(d) => {
            let ts = chrono::Utc::now().timestamp();
            let _ = write!(key, "{uid}:{session}:PermissionRequest:{}:{ts}", d.tool_name);
        }
        HookEvent::UserPromptSubmit(d) => {
            let h = Sha256::digest(d.prompt.as_bytes());
            let prompt_hash = hex::encode(&h[..8]);
            let _ = write!(key, "{uid}:{session}:UserPromptSubmit:{prompt_hash}");
        }
        HookEvent::SessionStart(_) | HookEvent::SessionEnd(_) => {
            let _ = write!(key, "{uid}:{session}:{}", payload.event_name());
        }
        HookEvent::TaskCompleted(d) => {
            let _ = write!(key, "{uid}:{session}:TaskCompleted:{}", d.task_id);
        }
        HookEvent::SubagentStop(_) => {
            let agent_id = payload.common.agent_id.as_deref().unwrap_or("");
            let _ = write!(key, "{uid}:{session}:SubagentStop:{agent_id}");
        }
        HookEvent::SubagentStart(_) => {
            let agent_id = payload.common.agent_id.as_deref().unwrap_or("");
            let _ = write!(key, "{uid}:{session}:SubagentStart:{agent_id}");
        }
        HookEvent::Stop(_) => {
            let ts = chrono::Utc::now().timestamp();
            let _ = write!(key, "{uid}:{session}:Stop:{ts}");
        }
        HookEvent::TeammateIdle(d) => {
            let ts = chrono::Utc::now().timestamp();
            let _ = write!(key, "{uid}:{session}:TeammateIdle:{}:{ts}", d.teammate_name);
        }
        HookEvent::Notification(_)
        | HookEvent::ConfigChange(_)
        | HookEvent::WorktreeCreate(_)
        | HookEvent::WorktreeRemove(_)
        | HookEvent::PreCompact(_)
        | HookEvent::InstructionsLoaded(_)
        | HookEvent::Unknown(_) => {
            let _ = write!(key, "{}", uuid::Uuid::new_v4());
        }
    }

    key
}

fn write_post_tool_use(
    key: &mut String,
    uid: &str,
    session: &str,
    d: &crate::types::webhook::PostToolUseData,
) {
    if d.tool_use_id.is_empty() {
        let ts = chrono::Utc::now().timestamp();
        let _ = write!(key, "{uid}:{session}:PostToolUse:{}:{ts}", d.tool_name);
    } else {
        let _ = write!(key, "{uid}:{session}:PostToolUse:{}", d.tool_use_id);
    }
}

fn write_post_tool_failure(
    key: &mut String,
    uid: &str,
    session: &str,
    d: &crate::types::webhook::PostToolUseFailureData,
) {
    if d.tool_use_id.is_empty() {
        let ts = chrono::Utc::now().timestamp();
        let _ = write!(key, "{uid}:{session}:PostToolUseFailure:{}:{ts}", d.tool_name);
    } else {
        let _ = write!(key, "{uid}:{session}:PostToolUseFailure:{}", d.tool_use_id);
    }
}
