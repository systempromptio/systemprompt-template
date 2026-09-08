//! View-model builders for the two judgement panels on the session detail
//! page: the stored AI analysis, and the human ratings people left.

use crate::handlers::ssr::format::{local_time, short_id};
use crate::repositories::analytics::session_quality::{SessionAnalysisSummary, SessionRatingRow};

use super::context::{AnalysisView, RatingView};

pub(super) fn analysis_view(a: &SessionAnalysisSummary) -> AnalysisView {
    let tags = a.tags.clone();
    AnalysisView {
        title: a.title.clone(),
        summary: a.summary.clone(),
        category: a.category.clone(),
        outcome: a.outcome.clone(),
        goal_achieved: a.goal_achieved.clone(),
        quality_score: a.quality_score,
        quality_tone: score_tone(a.quality_score),
        goal_tone: goal_tone(&a.goal_achieved),
        has_tags: !tags.is_empty(),
        tags,
        recommendations: non_empty(a.recommendations.as_deref()),
        improvement_hints: non_empty(a.improvement_hints.as_deref()),
        error_analysis: non_empty(a.error_analysis.as_deref()),
        corrections_count: a.corrections_count,
        duration_display: a
            .session_duration_minutes
            .map_or_else(|| "—".to_owned(), |m| format!("{m} min")),
        turns_display: a
            .total_turns
            .map_or_else(|| "—".to_owned(), |t| t.to_string()),
        updated_at_local: local_time(a.updated_at),
    }
}

pub(super) fn rating_view(r: &SessionRatingRow) -> RatingView {
    let user_label = r
        .display_name
        .clone()
        .unwrap_or_else(|| short_id(r.user_id.as_str()));
    RatingView {
        user_url: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
        user_id: r.user_id.clone(),
        user_label,
        rating: r.rating,
        stars: stars(r.rating),
        tone: score_tone(r.rating * 20),
        outcome: if r.outcome.is_empty() {
            "—".to_owned()
        } else {
            r.outcome.clone()
        },
        notes: if r.notes.is_empty() {
            "—".to_owned()
        } else {
            r.notes.clone()
        },
        created_at_local: local_time(r.created_at),
    }
}

// Why: Mean of the ratings, to one decimal. `None` when nobody has rated yet,
// so the panel shows the count only rather than an average of nothing.
pub(super) fn rating_average(ratings: &[SessionRatingRow]) -> Option<String> {
    if ratings.is_empty() {
        return None;
    }
    let sum: i64 = ratings.iter().map(|r| i64::from(r.rating)).sum();
    let mean = sum as f64 / ratings.len() as f64;
    Some(format!("{mean:.1}"))
}

// Why: the analysis writer scores 0-100 and a human rates 1-5; both are mapped
// onto the same three tones so the two panels read on one scale.
const fn score_tone(score: i16) -> &'static str {
    match score {
        s if s >= 80 => "ok",
        s if s >= 50 => "warn",
        _ => "err",
    }
}

fn goal_tone(goal: &str) -> &'static str {
    match goal.to_lowercase().as_str() {
        "yes" | "true" | "achieved" | "complete" => "ok",
        "partial" | "partially" => "warn",
        _ => "err",
    }
}

fn stars(rating: i16) -> String {
    let filled = rating.clamp(0, 5);
    let empty = 5 - filled;
    "★".repeat(filled as usize) + &"☆".repeat(empty as usize)
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty()).map(str::to_owned)
}
