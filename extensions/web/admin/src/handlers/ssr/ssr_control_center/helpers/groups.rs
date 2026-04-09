use std::collections::{HashMap, HashSet};

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::control_center;
use crate::repositories::session_analyses::SessionAnalysisRow;

use super::super::session_groups::build_session_groups_with_status;
use super::super::types::SessionGroup;

pub async fn build_session_groups(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    recent_sessions: &[crate::types::control_center::RecentSession],
) -> Vec<SessionGroup> {
    let session_ids: Vec<String> = recent_sessions
        .iter()
        .map(|s| s.session_id.clone())
        .collect();
    let (activity_feed, analyses_batch) = tokio::join!(
        control_center::fetch_session_events(pool, user_id, &session_ids),
        repositories::session_analyses::fetch_session_analyses_batch(pool, &session_ids),
    );
    let activity_feed = activity_feed.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch session activity feed");
        vec![]
    });

    let active_sessions: HashSet<String> = recent_sessions
        .iter()
        .filter(|s| s.ended_at.is_none() && s.status == "active")
        .map(|s| s.session_id.clone())
        .collect();
    let status_map: HashMap<String, String> = recent_sessions
        .iter()
        .map(|s| (s.session_id.clone(), s.status.clone()))
        .collect();

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
    for group in groups.iter_mut() {
        if let Some(analysis) = analyses_batch
            .iter()
            .find(|a| a.session_id == group.session_id)
        {
            group.flags.is_analysed = true;
            group.ai_summary = Some(analysis.summary.clone());
            group.ai_tags = Some(analysis.tags.clone());
            if !analysis.title.is_empty() {
                group.ai_title = Some(analysis.title.clone());
            }
            group.quality_score = analysis.quality_score;
            group.goal_achieved = analysis.goal_achieved.clone();
            group.quality_class = match analysis.quality_score {
                4..=5 => "high",
                3 => "medium",
                _ => "low",
            }
            .to_string();
            group.goal_icon = match analysis.goal_achieved.as_str() {
                "yes" => "\u{2713}",
                "partial" => "\u{25CF}",
                "no" => "\u{2717}",
                _ => "",
            }
            .to_string();
            if !analysis.description.is_empty() {
                group.description = Some(analysis.description.clone());
            }
            let has_recs = analysis
                .recommendations
                .as_ref()
                .is_some_and(|r| !r.is_empty());
            if has_recs {
                group.recommendations = analysis.recommendations.clone();
            }
        } else {
            group.flags.is_analysed = false;
        }

        if let Some(session) = recent_sessions
            .iter()
            .find(|s| s.session_id == group.session_id)
        {
            group.content_bytes = session.content_input_bytes + session.content_output_bytes;

            let source = &session.client_source;
            group.client_source = source.clone();
            group.client_source_label = format_client_source(source).to_string();
            group.client_source_class = client_source_class(source).to_string();

            let mode = &session.permission_mode;
            group.permission_mode = mode.clone();
            group.flags.is_plan_mode = mode == "plan";

            group.model = session.model.clone();
            group.model_short = format_model_short(&session.model);

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
    let skills: Vec<_> = entity_usage
        .iter()
        .filter(|e| e.entity_type == "skill")
        .collect();
    let agents: Vec<_> = entity_usage
        .iter()
        .filter(|e| e.entity_type == "agent")
        .collect();
    let mcp: Vec<_> = entity_usage
        .iter()
        .filter(|e| e.entity_type == "mcp_tool")
        .collect();
    (skills, agents, mcp)
}
