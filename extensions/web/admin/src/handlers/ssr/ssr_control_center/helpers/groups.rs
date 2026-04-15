use std::collections::{HashMap, HashSet};

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::control_center;
use crate::repositories::session_analyses::SessionAnalysisRow;
use crate::types::{ENTITY_AGENT, ENTITY_MCP_TOOL, ENTITY_SKILL, PERMISSION_MODE_PLAN, STATUS_ACTIVE};

use super::super::session_groups::build_session_groups_with_status;
use super::super::types::SessionGroup;

pub async fn build_session_groups(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    recent_sessions: &[crate::types::control_center::RecentSession],
) -> Vec<SessionGroup> {
    let mut session_ids = Vec::with_capacity(recent_sessions.len());
    let mut active_sessions = HashSet::new();
    let mut status_map = HashMap::with_capacity(recent_sessions.len());
    for s in recent_sessions {
        session_ids.push(s.session_id.clone());
        if s.ended_at.is_none() && s.status == STATUS_ACTIVE {
            active_sessions.insert(s.session_id.clone());
        }
        status_map.insert(s.session_id.clone(), s.status.clone());
    }

    let (activity_feed, analyses_batch) = tokio::join!(
        control_center::fetch_session_events(pool, user_id, &session_ids),
        repositories::session_analyses::fetch_session_analyses_batch(pool, &session_ids),
    );
    let activity_feed = activity_feed.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch session activity feed");
        vec![]
    });

    let mut groups =
        build_session_groups_with_status(&activity_feed, &active_sessions, &status_map);

    inject_analysis_data(&mut groups, &analyses_batch, recent_sessions);

    groups
}

fn inject_analysis_data(
    groups: &mut [SessionGroup],
    analyses_batch: &[SessionAnalysisRow],
    recent_sessions: &[crate::types::control_center::RecentSession],
) {
    let analyses_map: HashMap<String, &SessionAnalysisRow> = analyses_batch
        .iter()
        .map(|a| (a.session_id.clone(), a))
        .collect();
    let sessions_map: HashMap<String, &crate::types::control_center::RecentSession> =
        recent_sessions
            .iter()
            .map(|s| (s.session_id.clone(), s))
            .collect();

    for group in groups.iter_mut() {
        if let Some(analysis) = analyses_map.get(&group.session_id) {
            group.flags.is_analysed = true;
            group.ai_summary = Some(analysis.summary.clone());
            group.ai_tags = Some(analysis.tags.clone());
            if !analysis.title.is_empty() {
                group.ai_title = Some(analysis.title.clone());
            }
            group.quality_score = analysis.quality_score;
            group.quality_class = match analysis.quality_score {
                4..=5 => "high",
                3 => "medium",
                _ => "low",
            }
            .to_string();
            group.goal_achieved = analysis.goal_achieved.clone();
            group.goal_icon = match group.goal_achieved.as_str() {
                "yes" => "\u{2713}",
                "partial" => "\u{25CF}",
                "no" => "\u{2717}",
                _ => "",
            }
            .to_string();
            if !analysis.description.is_empty() {
                group.description = Some(analysis.description.clone());
            }
            if analysis
                .recommendations
                .as_ref()
                .is_some_and(|r| !r.is_empty())
            {
                group.recommendations = analysis.recommendations.clone();
            }
        } else {
            group.flags.is_analysed = false;
        }

        if let Some(session) = sessions_map.get(&group.session_id) {
            group.content_bytes = session.content_input_bytes + session.content_output_bytes;

            group.client_source_label =
                format_client_source(&session.client_source).to_string();
            group.client_source_class =
                client_source_class(&session.client_source).to_string();
            group.client_source = session.client_source.clone();

            group.flags.is_plan_mode = session.permission_mode == PERMISSION_MODE_PLAN;
            group.permission_mode = session.permission_mode.clone();

            group.model_short = format_model_short(&session.model);
            group.model = session.model.clone();

            group.user_prompts = session.user_prompts;
            group.automated_actions = session.automated_actions;
        }
    }
}

fn format_client_source(source: &str) -> &str {
    match source {
        "cli" => "CLI",
        "vscode" => "VS Code",
        "jetbrains" => "JetBrains",
        "claude-desktop" | "desktop" => "Desktop",
        "claude-code-desktop" => "CC Desktop",
        "" => "",
        other => other,
    }
}

fn client_source_class(source: &str) -> &str {
    match source {
        "cli" => "cli",
        "vscode" => "vscode",
        "jetbrains" => "jetbrains",
        "claude-desktop" | "desktop" | "claude-code-desktop" => "desktop",
        _ => "other",
    }
}

fn format_model_short(model: &str) -> String {
    if model.contains("opus") {
        "Opus".to_string()
    } else if model.contains("sonnet") {
        "Sonnet".to_string()
    } else if model.contains("haiku") {
        "Haiku".to_string()
    } else if model.is_empty() {
        String::new()
    } else {
        model.rsplit('-').next().unwrap_or(model).to_string()
    }
}

pub fn partition_entity_usage(
    entity_usage: &[crate::types::conversation_analytics::EntityUsageSummary],
) -> (
    Vec<&crate::types::conversation_analytics::EntityUsageSummary>,
    Vec<&crate::types::conversation_analytics::EntityUsageSummary>,
    Vec<&crate::types::conversation_analytics::EntityUsageSummary>,
) {
    let mut skills = Vec::new();
    let mut agents = Vec::new();
    let mut mcp = Vec::new();
    for entry in entity_usage {
        match entry.entity_type.as_str() {
            ENTITY_SKILL => skills.push(entry),
            ENTITY_AGENT => agents.push(entry),
            ENTITY_MCP_TOOL => mcp.push(entry),
            _ => {}
        }
    }
    (skills, agents, mcp)
}
