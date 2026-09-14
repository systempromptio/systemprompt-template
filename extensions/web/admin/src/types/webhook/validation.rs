//! Validation of identity and event fields at the hook boundary.

use super::{HookEvent, HookEventPayload};

pub fn validate_session_key(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 255
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_:.".contains(&c))
    {
        return Err("session_id must be a nonempty identifier of at most 255 bytes".to_owned());
    }
    Ok(())
}

impl HookEventPayload {
    pub fn validate_ingestion(&self) -> Result<(), String> {
        validate_session_key(self.session_id())?;
        if self.common.hook_event_name.is_empty() || matches!(self.event, HookEvent::Unknown(_)) {
            return Err("Unsupported or malformed hook event".to_owned());
        }
        match &self.event {
            HookEvent::UserPromptSubmit(d) if d.prompt.trim().is_empty() => {
                return Err("UserPromptSubmit requires prompt".to_owned());
            },
            HookEvent::PreToolUse(d) if d.name.is_empty() || d.use_id.is_empty() => {
                return Err("PreToolUse requires tool_name and tool_use_id".to_owned());
            },
            HookEvent::PostToolUse(d) if d.name.is_empty() || d.use_id.is_empty() => {
                return Err("PostToolUse requires tool_name and tool_use_id".to_owned());
            },
            HookEvent::PostToolUseFailure(d)
                if d.tool_name.is_empty() || d.tool_use_id.is_empty() =>
            {
                return Err("PostToolUseFailure requires tool_name and tool_use_id".to_owned());
            },
            _ => {},
        }
        Ok(())
    }

    pub fn delivery_id(&self) -> Result<&str, String> {
        let key = match self.event {
            HookEvent::UserPromptSubmit(_) => "prompt_id",
            HookEvent::PreToolUse(_)
            | HookEvent::PostToolUse(_)
            | HookEvent::PostToolUseFailure(_) => "tool_use_id",
            _ => "event_id",
        };
        // JSON: the hook envelope is an external protocol with event-specific keys.
        self.raw
            .get(key)
            .and_then(serde_json::Value::as_str)
            .or_else(|| self.raw.get("event_id").and_then(serde_json::Value::as_str))
            .filter(|v| !v.is_empty() && v.len() <= 255 && !v.chars().any(char::is_control))
            .ok_or_else(|| format!("{key} or event_id is required for retry-safe ingestion"))
    }
}
