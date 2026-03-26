use std::sync::Arc;

use sqlx::PgPool;

use crate::admin::activity::{self, NewActivity};
use crate::admin::repositories;
use crate::admin::types::HookEventPayload;

pub(super) struct ActivityRecordingParams<'a> {
    pub pool: &'a Arc<PgPool>,
    pub user_id: &'a str,
    pub event_type: &'a str,
    pub session_id: &'a str,
    pub plugin_id: Option<&'a str>,
    pub payload: &'a HookEventPayload,
}

pub(super) fn spawn_activity_recording(params: &ActivityRecordingParams<'_>) {
    if params.event_type == "claude_code_SessionStart" {
        spawn_plugin_installation_tracking(params);
    }

    let extra = &params.payload.extra;
    let activity = match params.event_type {
        "claude_code_SessionStart" => {
            let model = params.payload.model.as_deref().unwrap_or("unknown");
            let project = params.payload.project_path.as_deref();
            let source = params.payload.source.as_deref().or_else(|| extra.get("source").and_then(|v| v.as_str()));
            NewActivity::session_started_rich(params.user_id, params.session_id, model, project, source)
        }
        "claude_code_SessionEnd" => {
            let reason = params.payload.reason.as_deref().or_else(|| extra.get("reason").and_then(|v| v.as_str()));
            NewActivity::session_ended_rich(params.user_id, params.session_id, reason)
        }
        "claude_code_UserPromptSubmit" => {
            let prompt = params.payload.prompt.as_deref().or_else(|| extra.get("prompt").and_then(|v| v.as_str()));
            NewActivity::prompt_submitted_rich(params.user_id, params.session_id, prompt)
        }
        "claude_code_PostToolUse" => {
            let tool = params.payload.tool_name.as_deref().unwrap_or("unknown");
            let tool_input = params.payload.tool_input.as_ref().or_else(|| extra.get("tool_input"));
            let detail = extract_tool_detail(tool, tool_input);
            if is_internal_tool(tool) {
                NewActivity::tool_used(params.user_id, tool, params.session_id, &detail)
            } else {
                NewActivity::skill_used_rich(params.user_id, tool, params.session_id, &detail)
            }
        }
        "claude_code_PostToolUseFailure" => {
            let tool = params.payload.tool_name.as_deref().unwrap_or("unknown");
            let error = params.payload.error.as_deref().or_else(|| extra.get("error").and_then(|v| v.as_str()));
            NewActivity::tool_error(params.user_id, tool, params.session_id, error)
        }
        "claude_code_Stop" => {
            let msg = params.payload.last_assistant_message.as_deref().or_else(|| extra.get("last_assistant_message").and_then(|v| v.as_str()));
            NewActivity::agent_response(params.user_id, params.session_id, msg)
        }
        "claude_code_SubagentStart" => {
            let agent_type = params.payload.agent_type.as_deref().or_else(|| extra.get("agent_type").and_then(|v| v.as_str()));
            NewActivity::subagent_started(params.user_id, params.session_id, agent_type)
        }
        "claude_code_Notification" => {
            let ntype = extra.get("notification_type").and_then(|v| v.as_str());
            if ntype != Some("permission_prompt") {
                return;
            }
            let message = extra.get("message").and_then(|v| v.as_str());
            NewActivity::notification(params.user_id, params.session_id, ntype, message)
        }
        "claude_code_TaskCompleted" => {
            let subject = extra.get("task_subject").and_then(|v| v.as_str());
            NewActivity::task_completed_activity(params.user_id, params.session_id, subject)
        }
        "claude_code_PreCompact" => {
            let trigger = extra.get("trigger").and_then(|v| v.as_str());
            NewActivity::context_compacted(params.user_id, params.session_id, trigger)
        }
        _ => return,
    };

    let p = params.pool.clone();
    tokio::spawn(async move {
        activity::record(&p, activity).await;
    });
}

fn spawn_plugin_installation_tracking(params: &ActivityRecordingParams<'_>) {
    let plugin_version = params.payload.extra.get("plugin_version").and_then(|v| v.as_str());
    if let (Some(pid), Some(version)) = (params.plugin_id, plugin_version) {
        let base_plugin_id = params.payload.extra.get("base_plugin_id").and_then(|v| v.as_str());
        let plugin_source = if base_plugin_id.is_some() { "custom" } else { "org" };
        let p = params.pool.clone();
        let uid = params.user_id.to_string();
        let pid = pid.to_string();
        let ver = version.to_string();
        let src = plugin_source.to_string();
        let base = base_plugin_id.map(str::to_string);
        tokio::spawn(async move {
            match repositories::upsert_plugin_installation(
                p.as_ref(), &uid, &pid, &ver, &src, base.as_deref(),
            ).await {
                Ok(event) => {
                    tracing::info!(user_id = %uid, plugin_id = %pid, version = %ver, source = %src, event = %event, "Plugin installation tracked");
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to track plugin installation");
                }
            }
        });
    }
}

fn extract_tool_detail(tool: &str, tool_input: Option<&serde_json::Value>) -> String {
    let Some(input) = tool_input else {
        return format!("used {tool}");
    };
    match tool {
        "Bash" => input.get("command").and_then(|v| v.as_str())
            .map_or_else(|| "ran a command".into(), |cmd| {
                let first_line = cmd.lines().next().unwrap_or(cmd);
                if first_line.len() > 60 {
                    format!("{}...", &first_line[..57])
                } else {
                    first_line.to_string()
                }
            }),
        "Read" => input.get("file_path").and_then(|v| v.as_str())
            .map_or_else(|| "read a file".into(), |p| format!("read {}", short_path(p))),
        "Write" => input.get("file_path").and_then(|v| v.as_str())
            .map_or_else(|| "wrote a file".into(), |p| format!("wrote {}", short_path(p))),
        "Edit" | "MultiEdit" => input.get("file_path").and_then(|v| v.as_str())
            .map_or_else(|| "edited a file".into(), |p| format!("edited {}", short_path(p))),
        "Grep" => input.get("pattern").and_then(|v| v.as_str())
            .map_or_else(|| "searched files".into(), |pat| {
                let t = if pat.len() > 40 { format!("{}...", &pat[..37]) } else { pat.to_string() };
                format!("searched for '{t}'")
            }),
        "Glob" => input.get("pattern").and_then(|v| v.as_str())
            .map_or_else(|| "found files".into(), |pat| {
                let t = if pat.len() > 40 { format!("{}...", &pat[..37]) } else { pat.to_string() };
                format!("found '{t}'")
            }),
        "WebSearch" => input.get("query").and_then(|v| v.as_str())
            .map_or_else(|| "searched web".into(), |q| {
                let t = if q.len() > 50 { format!("{}...", &q[..47]) } else { q.to_string() };
                format!("searched: '{t}'")
            }),
        "WebFetch" => input.get("url").and_then(|v| v.as_str())
            .map_or_else(|| "fetched URL".into(), |u| {
                let t = if u.len() > 50 { format!("{}...", &u[..47]) } else { u.to_string() };
                format!("fetched {t}")
            }),
        "Task" => input.get("description").and_then(|v| v.as_str())
            .map_or_else(|| "spawned agent".into(), |d| {
                let t = if d.len() > 50 { format!("{}...", &d[..47]) } else { d.to_string() };
                format!("spawned: {t}")
            }),
        "Skill" => input.get("skill").and_then(|v| v.as_str())
            .map_or_else(|| "invoked skill".into(), |s| format!("invoked /{s}")),
        _ => format!("used {tool}"),
    }
}

fn is_internal_tool(tool: &str) -> bool {
    matches!(tool,
        "Bash" | "Read" | "Write" | "Edit" | "Glob" | "Grep" |
        "NotebookEdit" | "Task" | "TodoRead" | "TodoWrite" |
        "WebFetch" | "WebSearch" | "AskFollowupQuestion" |
        "AttemptCompletion" | "MultiEdit" | "AskUserQuestion" |
        "EnterPlanMode" | "ExitPlanMode" | "Skill"
    )
}

fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.rsplit('/').take(2).collect();
    if parts.len() == 2 {
        format!("{}/{}", parts[1], parts[0])
    } else {
        (*parts.first().unwrap_or(&path)).to_string()
    }
}
