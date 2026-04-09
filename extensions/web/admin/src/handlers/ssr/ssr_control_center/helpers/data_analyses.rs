use crate::repositories::session_analyses::SessionAnalysisRow;

use super::super::types::AnalysisEntry;
use super::metrics::parse_summary_parts;

pub(in crate::admin) struct AnalysesData {
    pub recent_analyses_json: Vec<AnalysisEntry>,
    pub recommendations_json: Vec<AnalysisEntry>,
}

pub(in crate::admin) fn build_analyses_data(
    recent_analyses: &[SessionAnalysisRow],
) -> AnalysesData {
    let recent_analyses_json: Vec<AnalysisEntry> = recent_analyses
        .iter()
        .map(|a| {
            let quality_class = match a.quality_score {
                4..=5 => "high",
                3 => "medium",
                _ => "low",
            };
            let (goal_summary, outcomes) = parse_summary_parts(&a.summary);
            let tags_list: Vec<String> = a
                .tags
                .split(',')
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(String::from)
                .collect();
            AnalysisEntry {
                session_id: a.session_id.clone(),
                title: a.title.clone(),
                description: a.description.clone(),
                goal_summary,
                outcomes,
                tags: a.tags.clone(),
                tags_list,
                goal_achieved: a.goal_achieved.clone(),
                quality_score: a.quality_score,
                quality_class,
                outcome: a.outcome.clone(),
                error_analysis: a.error_analysis.clone(),
                skill_assessment: a.skill_assessment.clone(),
                recommendations: a.recommendations.clone(),
                category: a.category.clone(),
                goal_outcome_map: a.goal_outcome_map.clone(),
                efficiency_metrics: a.efficiency_metrics.clone(),
                best_practices_checklist: a.best_practices_checklist.clone(),
                improvement_hints: a.improvement_hints.clone(),
                corrections_count: a.corrections_count,
                total_turns: a.total_turns,
                session_duration_minutes: a.session_duration_minutes,
            }
        })
        .collect();

    let recommendations_json: Vec<AnalysisEntry> = recent_analyses_json
        .iter()
        .filter(|a| a.recommendations.as_ref().is_some_and(|r| !r.is_empty()))
        .cloned()
        .collect();

    AnalysesData {
        recent_analyses_json,
        recommendations_json,
    }
}

pub(in crate::admin) struct SkillAdoption {
    pub adoption_pct: usize,
    pub total_available: usize,
    pub total_used: usize,
}

pub(in crate::admin) const fn build_skill_adoption(
    skills_usage: &[&crate::types::conversation_analytics::EntityUsageSummary],
    unused_skills: &[String],
) -> SkillAdoption {
    let total_available = skills_usage.len() + unused_skills.len();
    let total_used = skills_usage.len();
    let adoption_pct = if total_available > 0 {
        (total_used * 100) / total_available
    } else {
        0
    };
    SkillAdoption {
        adoption_pct,
        total_available,
        total_used,
    }
}
