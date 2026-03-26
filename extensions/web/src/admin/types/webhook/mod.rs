mod event_types;
mod parsers;

pub use event_types::*;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct HookEventPayload {
    pub common: HookCommonFields,
    pub event: HookEvent,
    pub raw: serde_json::Value,
}

impl HookEventPayload {
    #[must_use]
    pub fn from_value(raw: serde_json::Value) -> (Self, Vec<String>) {
        let mut warnings = Vec::new();
        let common = parse_common_fields(&raw, &mut warnings);
        let event_name = resolve_event_name(&common);
        let event = dispatch_event(event_name, &raw, &common, &mut warnings);
        (Self { common, event, raw }, warnings)
    }

    #[must_use]
    pub fn event_name(&self) -> &str {
        match &self.event {
            HookEvent::SessionStart(_) => "SessionStart",
            HookEvent::SessionEnd(_) => "SessionEnd",
            HookEvent::UserPromptSubmit(_) => "UserPromptSubmit",
            HookEvent::PreToolUse(_) => "PreToolUse",
            HookEvent::PostToolUse(_) => "PostToolUse",
            HookEvent::PostToolUseFailure(_) => "PostToolUseFailure",
            HookEvent::PermissionRequest(_) => "PermissionRequest",
            HookEvent::Stop(_) => "Stop",
            HookEvent::SubagentStart(_) => "SubagentStart",
            HookEvent::SubagentStop(_) => "SubagentStop",
            HookEvent::TaskCompleted(_) => "TaskCompleted",
            HookEvent::TeammateIdle(_) => "TeammateIdle",
            HookEvent::Notification(_) => "Notification",
            HookEvent::ConfigChange(_) => "ConfigChange",
            HookEvent::WorktreeCreate(_) => "WorktreeCreate",
            HookEvent::WorktreeRemove(_) => "WorktreeRemove",
            HookEvent::PreCompact(_) => "PreCompact",
            HookEvent::InstructionsLoaded(_) => "InstructionsLoaded",
            HookEvent::Unknown(name) => name.as_str(),
        }
    }

    #[must_use]
    pub fn tool_name(&self) -> Option<&str> {
        match &self.event {
            HookEvent::PreToolUse(d) if !d.tool_name.is_empty() => Some(&d.tool_name),
            HookEvent::PostToolUse(d) if !d.tool_name.is_empty() => Some(&d.tool_name),
            HookEvent::PostToolUseFailure(d) if !d.tool_name.is_empty() => Some(&d.tool_name),
            HookEvent::PermissionRequest(d) if !d.tool_name.is_empty() => Some(&d.tool_name),
            _ => None,
        }
    }

    #[must_use]
    pub fn tool_input(&self) -> Option<&serde_json::Value> {
        match &self.event {
            HookEvent::PostToolUse(d) => Some(&d.tool_input),
            HookEvent::PostToolUseFailure(d) => Some(&d.tool_input),
            _ => None,
        }
    }

    #[must_use]
    pub fn model(&self) -> Option<&str> {
        match &self.event {
            HookEvent::SessionStart(d) if !d.model.is_empty() => Some(&d.model),
            _ => None,
        }
    }

    #[must_use]
    pub fn cwd(&self) -> Option<&str> {
        if self.common.cwd.is_empty() {
            None
        } else {
            Some(&self.common.cwd)
        }
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.common.session_id
    }
}

fn parse_common_fields(raw: &serde_json::Value, warnings: &mut Vec<String>) -> HookCommonFields {
    tracing::debug!(raw_keys = ?raw.as_object().map(|o| o.keys().collect::<Vec<_>>()), "Hook payload keys");
    let common: HookCommonFields = serde_json::from_value(raw.clone()).unwrap_or_else(|e| {
        warnings.push(format!("Failed to parse common fields: {e}"));
        HookCommonFields {
            session_id: String::new(),
            cwd: String::new(),
            permission_mode: String::new(),
            transcript_path: String::new(),
            hook_event_name: String::new(),
            agent_id: None,
            agent_type: None,
        }
    });
    tracing::debug!(
        parsed_session_id = %common.session_id,
        parsed_hook_event = %common.hook_event_name,
        parsed_cwd = %common.cwd,
        "Hook common fields parsed"
    );

    if common.session_id.is_empty() {
        warnings.push("Missing required common field: session_id".to_string());
    }
    if common.cwd.is_empty() {
        warnings.push("Missing required common field: cwd".to_string());
    }

    common
}

fn resolve_event_name(common: &HookCommonFields) -> &str {
    if common.hook_event_name.is_empty() {
        "unknown"
    } else {
        common.hook_event_name.as_str()
    }
}

fn dispatch_event(
    event_name: &str,
    raw: &serde_json::Value,
    common: &HookCommonFields,
    warnings: &mut Vec<String>,
) -> HookEvent {
    match event_name {
        "SessionStart" => parsers::parse_session_start(raw, warnings),
        "SessionEnd" => parsers::parse_session_end(raw, warnings),
        "UserPromptSubmit" => parsers::parse_user_prompt_submit(raw, warnings),
        "PreToolUse" => parsers::parse_pre_tool_use(raw, warnings),
        "PostToolUse" => parsers::parse_post_tool_use(raw, warnings),
        "PostToolUseFailure" => parsers::parse_post_tool_use_failure(raw, warnings),
        "PermissionRequest" => parsers::parse_permission_request(raw, warnings),
        "Stop" => parsers::parse_stop(raw, warnings),
        "SubagentStart" => parsers::parse_subagent_start(raw, common, warnings),
        "SubagentStop" => parsers::parse_subagent_stop(raw, common, warnings),
        "TaskCompleted" => parsers::parse_task_completed(raw, warnings),
        "TeammateIdle" => parsers::parse_teammate_idle(raw, warnings),
        "Notification" => parsers::parse_notification(raw, warnings),
        "ConfigChange" => parsers::parse_config_change(raw, warnings),
        "WorktreeCreate" => parsers::parse_worktree_create(raw, warnings),
        "WorktreeRemove" => parsers::parse_worktree_remove(raw, warnings),
        "PreCompact" => parsers::parse_pre_compact(raw, warnings),
        "InstructionsLoaded" => parsers::parse_instructions_loaded(raw, warnings),
        other => {
            warnings.push(format!("Unknown hook event type: {other}"));
            HookEvent::Unknown(other.to_string())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TrackQuery {
    pub plugin_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SkillTrackQuery {
    pub sid: String,
    pub pid: Option<String>,
}

// --- foodles-specific types ---

#[derive(Debug, Deserialize)]
pub struct GovernQuery {
    pub plugin_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLinePayload {
    pub model: Option<StatusLineModel>,
    pub cost: Option<StatusLineCost>,
    pub context_window: Option<ContextWindow>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct StatusLineModel {
    pub api_model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLineCost {
    pub total_cost_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ContextWindow {
    pub context_window_size: Option<i64>,
    pub current_usage: Option<ContextWindowUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct ContextWindowUsage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLineQuery {
    pub plugin_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptPayload {
    pub session_id: Option<String>,
    pub transcript: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptQuery {
    pub plugin_id: Option<String>,
}
