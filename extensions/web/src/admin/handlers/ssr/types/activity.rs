use serde::Serialize;

use super::gamification::{AchievementCategoryView, EnrichedAchievementView};

#[derive(Debug, Clone, Serialize)]
pub struct MyActivityPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub events: Vec<crate::admin::activity::types::ActivityTimelineEvent>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_offset: i64,
    pub next_offset: i64,
    pub search: Option<String>,
    pub category_summary: Vec<crate::admin::activity::types::ActivityCategorySummary>,
    pub gamification: Option<crate::admin::types::UserGamificationProfile>,
    pub enriched_achievements: Vec<EnrichedAchievementView>,
    pub achievements_count: usize,
    pub achievements_by_category: Vec<AchievementCategoryView>,
    pub unlocked_achievements: usize,
    pub total_achievements: usize,
    pub total_activities: i64,
    pub total_edits: i64,
    pub total_sessions: i64,
    pub xp_progress_pct: u32,
}
