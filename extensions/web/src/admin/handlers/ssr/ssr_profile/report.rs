use std::sync::Arc;

use crate::admin::repositories::{daily_summaries, profile_reports};
use systemprompt::ai::AiService;

use super::analysis;
use super::archetype;

const PROFILE_REPORT_MAX_TOKENS: u32 = 65_536;

pub(super) const PROFILE_REPORT_PROMPT: &str = "\
You are an expert analyst of Claude Code usage patterns. \
Given a user's aggregate metrics over 30 days, their archetype classification, \
strengths, weaknesses, and comparison to global platform averages, \
produce a comprehensive profile analysis.";

fn profile_report_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "narrative": {
                "type": "string",
                "description": "3-5 paragraph comprehensive profile analysis covering how they use Claude Code, their style, and what makes them unique (max 2000 chars)"
            },
            "style_analysis": {
                "type": "string",
                "description": "How this user approaches Claude Code compared to typical users - their distinctive workflow patterns (max 500 chars)"
            },
            "comparison": {
                "type": "string",
                "description": "Specific metric comparisons to the average user with context on what the numbers mean (max 500 chars)"
            },
            "patterns": {
                "type": "string",
                "description": "Recurring behavioral patterns observed across their sessions - habits, preferences, tendencies (max 500 chars)"
            },
            "improvements": {
                "type": "string",
                "description": "Top 3 actionable improvements with specific metric targets they should aim for (max 500 chars)"
            },
            "tips": {
                "type": "string",
                "description": "3-5 personalized tips based on their archetype and weaknesses to level up their Claude Code usage (max 500 chars)"
            }
        },
        "required": ["narrative", "style_analysis", "comparison", "patterns", "improvements", "tips"]
    })
}

#[derive(serde::Deserialize)]
pub(super) struct AiProfileReport {
    pub(super) narrative: String,
    pub(super) style_analysis: String,
    pub(super) comparison: String,
    pub(super) patterns: String,
    pub(super) improvements: String,
    pub(super) tips: String,
}

pub(super) fn build_ai_context(
    user: &profile_reports::UserAggregateMetrics,
    global: &daily_summaries::GlobalAverages,
    archetype: &archetype::ArchetypeResult,
    strengths: &[analysis::MetricDeviation],
    weaknesses: &[analysis::MetricDeviation],
) -> String {
    let mut parts = Vec::new();

    parts.push(format!(
        "ARCHETYPE: {} (confidence: {}%)",
        archetype.name, archetype.confidence
    ));
    parts.push(format!(
        "Period: {} days, {} total sessions",
        user.total_days, user.total_sessions
    ));

    parts.push(format!(
        "USER METRICS: quality={:.1}, apm={:.1}, goal_rate={:.1}%, error_rate={:.1}%, tool_diversity={:.1}, multitasking={:.1}, sessions/day={:.1}",
        user.avg_quality, user.avg_apm, user.avg_goal_rate, user.avg_error_rate,
        user.avg_tool_diversity, user.avg_multitasking, user.avg_sessions_per_day,
    ));

    let g_q = global.avg_quality.unwrap_or(0.0);
    let g_a = global.avg_apm.unwrap_or(0.0);
    let g_goal = global.avg_goal_rate.unwrap_or(0.0);
    let g_err = global.avg_error_rate.unwrap_or(0.0);
    let g_td = global.avg_tool_diversity.unwrap_or(0.0);
    let g_m = global.avg_multitasking.unwrap_or(0.0);
    let g_s = global.avg_sessions.unwrap_or(0.0);
    parts.push(format!(
        "GLOBAL AVERAGES ({} users): quality={g_q:.1}, apm={g_a:.1}, goal_rate={g_goal:.1}%, error_rate={g_err:.1}%, tool_diversity={g_td:.1}, multitasking={g_m:.1}, sessions/day={g_s:.1}",

        global.total_users.unwrap_or(0),
    ));

    parts.push(format!(
        "TOTALS: prompts={}, tool_uses={}, errors={}, goals_achieved={}, goals_partial={}, goals_failed={}",
        user.total_prompts, user.total_tool_uses, user.total_errors,
        user.total_goals_achieved, user.total_goals_partial, user.total_goals_failed,
    ));

    if !strengths.is_empty() {
        let s: Vec<String> = strengths
            .iter()
            .map(|s| format!("{}: +{:.0}%", s.label, s.deviation_pct))
            .collect();
        parts.push(format!("STRENGTHS: {}", s.join(", ")));
    }

    if !weaknesses.is_empty() {
        let w: Vec<String> = weaknesses
            .iter()
            .map(|w| format!("{}: {:.0}%", w.label, w.deviation_pct))
            .collect();
        parts.push(format!("WEAKNESSES: {}", w.join(", ")));
    }

    if !user.category_distribution.is_empty() {
        let cats: Vec<String> = user
            .category_distribution
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        parts.push(format!("CATEGORIES: {}", cats.join(", ")));
    }

    parts.join("\n")
}

pub(super) async fn generate_ai_profile(
    ai: &Arc<AiService>,
    user_id: &str,
    context: &str,
) -> Option<AiProfileReport> {
    use systemprompt::ai::{AiMessage, AiRequest, StructuredOutputOptions};
    use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};
    use systemprompt::models::auth::UserType;
    use systemprompt::models::execution::context::RequestContext;

    let ctx = RequestContext::new(
        SessionId::new(format!("profile-{user_id}")),
        TraceId::new(uuid::Uuid::new_v4().to_string()),
        ContextId::new(""),
        AgentName::new("profile-report"),
    )
    .with_user_id(UserId::new(user_id))
    .with_user_type(UserType::User);

    let messages = vec![
        AiMessage::system(PROFILE_REPORT_PROMPT),
        AiMessage::user(context),
    ];

    let request = AiRequest::builder(
        messages,
        ai.default_provider(),
        ai.default_model(),
        PROFILE_REPORT_MAX_TOKENS,
        ctx,
    )
    .with_structured_output(StructuredOutputOptions::with_schema(profile_report_schema()))
    .build();

    let response = ai
        .generate(&request)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to generate AI profile report");
        })
        .ok()?;

    serde_json::from_str::<AiProfileReport>(&response.content)
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to parse AI profile report response");
        })
        .ok()
}
