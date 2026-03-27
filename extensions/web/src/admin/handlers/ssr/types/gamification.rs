use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AchievementView {
    pub achievement_id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub is_unlocked: bool,
    pub rarity_pct: f64,
    pub rarity_label: &'static str,
    pub total_unlocked: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlocked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AchievementCategoryView {
    pub category: &'static str,
    pub icon: &'static str,
    pub achievements: Vec<AchievementView>,
    pub unlocked_count: u32,
    pub total_count: u32,
    pub completion_pct: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RankView {
    pub level: i32,
    pub name: &'static str,
    pub xp: i64,
    pub is_current: bool,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AchievementsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub achievements_by_category: Vec<AchievementCategoryView>,
    pub profile: Option<crate::admin::types::UserGamificationProfile>,
    pub ranks: Vec<RankView>,
    pub can_scroll_left: bool,
    pub can_scroll_right: bool,
    pub current_rank_level: i32,
    pub max_rank: i32,
    pub xp_progress_pct: i64,
    pub unlocked_count: u32,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaderboardEntryView {
    pub position: usize,
    pub display_name: String,
    pub rank_level: i32,
    pub rank_name: String,
    pub total_xp: i64,
    pub events_count: i64,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub achievement_count: i64,
    pub total_sessions: i64,
    pub total_prompts: String,
    pub total_tool_uses: String,
    pub total_subagents: i64,
    pub unique_skills_count: i32,
    pub total_days_active: i32,
    pub is_self: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medal: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaderboardAveragesView {
    pub avg_xp: String,
    pub avg_sessions: String,
    pub avg_prompts: String,
    pub avg_tool_uses: String,
    pub avg_subagents: String,
    pub avg_streak: String,
    pub avg_achievements: String,
    pub avg_days_active: String,
    pub total_users: i64,
}

#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct LeaderboardPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub entries: Vec<LeaderboardEntryView>,
    pub podium: Vec<LeaderboardEntryView>,
    pub has_podium: bool,
    pub current_sort: String,
    pub sort_xp: bool,
    pub sort_sessions: bool,
    pub sort_prompts: bool,
    pub sort_tools: bool,
    pub sort_subagents: bool,
    pub sort_streak: bool,
    pub sort_achievements: bool,
    pub averages: Option<LeaderboardAveragesView>,
    pub has_averages: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrichedAchievementView {
    pub achievement_id: String,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub unlocked_at: chrono::DateTime<chrono::Utc>,
}
