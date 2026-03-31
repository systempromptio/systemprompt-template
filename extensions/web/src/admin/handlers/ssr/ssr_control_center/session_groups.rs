use std::collections::{HashMap, HashSet};

use crate::admin::types::control_center::ActivityFeedEvent;

use super::turns;
use super::types::{SessionGroup, SessionGroupFlags, ToolError, Turn};

pub fn build_session_groups_with_status(
    events: &[ActivityFeedEvent],
    active_session_ids: &HashSet<String>,
    status_map: &HashMap<String, String>,
) -> Vec<SessionGroup> {
    let meaningful: Vec<&ActivityFeedEvent> = events
        .iter()
        .filter(|e| !e.event_type.contains("PreToolUse"))
        .collect();

    let mut session_map: indexmap::IndexMap<String, Vec<&ActivityFeedEvent>> =
        indexmap::IndexMap::new();
    for evt in &meaningful {
        session_map
            .entry(evt.session_id.clone())
            .or_default()
            .push(evt);
    }

    let mut session_groups: Vec<SessionGroup> = Vec::new();

    for (session_id, events) in &session_map {
        let mut chronological: Vec<&&ActivityFeedEvent> = events.iter().collect();
        chronological.reverse();

        let group = build_single_group(session_id, &chronological, active_session_ids, status_map);
        session_groups.push(group);
    }

    session_groups.sort_by(|a, b| b.last_activity_at.cmp(&a.last_activity_at));

    session_groups
}

fn build_single_group(
    session_id: &str,
    chronological: &[&&ActivityFeedEvent],
    active_session_ids: &HashSet<String>,
    status_map: &HashMap<String, String>,
) -> SessionGroup {
    let project_name = extract_project_name(chronological);
    let is_active = active_session_ids.contains(session_id);
    let turn_list = turns::build_turns(chronological);
    let session_title = extract_session_title(chronological);
    let total_prompts = turns::count_prompts(&turn_list);
    let total_tools = turns::sum_tools(&turn_list);
    let total_errors = turns::sum_errors(&turn_list);

    let start_time = chronological.first().map(|e| e.created_at);
    let end_time = chronological.last().map(|e| e.created_at);
    let started_at = start_time.map_or_else(String::new, |t| t.to_rfc3339());
    let last_activity_at = end_time.map_or_else(String::new, |t| t.to_rfc3339());
    let duration_display = match (start_time, end_time) {
        (Some(s), Some(e)) => format_duration(e - s),
        _ => String::new(),
    };

    let status = resolve_status(session_id, status_map).to_string();
    let (first_prompt, last_response, all_errors) = extract_turn_details(&turn_list);
    let id_short = if session_id.len() > 8 {
        &session_id[..8]
    } else {
        session_id
    };

    let flags = SessionGroupFlags {
        is_active,
        ..SessionGroupFlags::default()
    };

    SessionGroup {
        session_id: session_id.to_string(),
        session_id_short: id_short.to_string(),
        project_name,
        session_title,
        started_at,
        last_activity_at,
        status,
        total_prompts,
        total_tools,
        total_errors,
        turn_count: turn_list.len(),
        duration_display,
        entity_count: 0,
        turns: turn_list,
        first_prompt,
        last_response,
        all_errors,
        flags,
        ..SessionGroup::default()
    }
}

fn extract_project_name(chronological: &[&&ActivityFeedEvent]) -> String {
    chronological
        .iter()
        .find_map(|e| {
            e.cwd
                .as_deref()
                .and_then(|p| p.rsplit('/').next())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or("unknown")
        .to_string()
}

fn resolve_status<'a>(session_id: &str, status_map: &'a HashMap<String, String>) -> &'a str {
    status_map.get(session_id).map_or("active", |v| v.as_str())
}

fn extract_turn_details(turn_list: &[Turn]) -> (String, String, Vec<ToolError>) {
    let first_prompt = turn_list
        .iter()
        .find_map(|t| {
            if t.prompt_text.is_empty() {
                None
            } else {
                Some(t.prompt_text.clone())
            }
        })
        .unwrap_or_else(String::new);

    let last_response = turn_list
        .iter()
        .rev()
        .find_map(|t| {
            if t.response_text.is_empty() {
                None
            } else {
                Some(t.response_text.clone())
            }
        })
        .unwrap_or_else(String::new);

    let all_errors: Vec<ToolError> = turn_list.iter().flat_map(|t| t.errors.clone()).collect();

    (first_prompt, last_response, all_errors)
}

fn format_duration(dur: chrono::Duration) -> String {
    let total_secs = dur.num_seconds().max(0);
    if total_secs < 60 {
        format!("{total_secs}s")
    } else if total_secs < 3600 {
        format!("{}m", total_secs / 60)
    } else if total_secs < 86400 {
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        if m > 0 {
            format!("{h}h {m}m")
        } else {
            format!("{h}h")
        }
    } else {
        let d = total_secs / 86400;
        let h = (total_secs % 86400) / 3600;
        if h > 0 {
            format!("{d}d {h}h")
        } else {
            format!("{d}d")
        }
    }
}

fn is_noise_title(s: &str) -> bool {
    const NOISE: &[&str] = &[
        "continue", "run", "yes", "no", "ok", "okay", "done", "stop", "help", "exit", "quit",
        "retry", "redo", "next", "go", "start", "y", "n",
    ];
    let lower = s.to_lowercase();
    let trimmed = lower.trim();
    NOISE.contains(&trimmed)
        || trimmed.starts_with("20")
        || trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit() && trimmed.contains(':'))
        || trimmed.starts_with("error:")
        || trimmed.starts_with("debug")
}

fn extract_session_title(chronological: &[&&ActivityFeedEvent]) -> String {
    chronological
        .iter()
        .filter(|e| e.event_type.contains("UserPromptSubmit"))
        .filter_map(|e| e.prompt_preview.as_deref())
        .filter(|s| {
            s.len() >= 10
                && !s.starts_with('<')
                && !s.starts_with('{')
                && !s.starts_with('"')
                && !s.starts_with("  ")
        })
        .map(|s| {
            let s = if s.starts_with('/') {
                s.find(' ').map_or("", |i| s[i..].trim_start())
            } else {
                s
            };
            if s.len() > 55 {
                let truncated = &s[..55];
                let end = truncated.rfind(' ').unwrap_or(55);
                format!("{}...", &s[..end])
            } else {
                s.to_string()
            }
        })
        .find(|s| s.len() >= 10 && !is_noise_title(s))
        .unwrap_or_else(String::new)
}
