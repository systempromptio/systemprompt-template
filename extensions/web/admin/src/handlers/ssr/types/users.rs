use serde::Serialize;

use super::gamification::EnrichedAchievementView;

#[derive(Debug, Clone, Serialize)]
pub struct EnrichedUserView {
    pub user_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub last_active: String,
    pub total_events: i64,
    pub last_tool: Option<String>,
    pub custom_skills_count: i64,
    pub preferred_client: Option<String>,
    pub prompts: i64,
    pub sessions: i64,
    pub bytes: i64,
    pub logins: i64,
    pub rank_name: String,
    pub xp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsersPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub users: Vec<EnrichedUserView>,
    pub total_users: usize,
    pub active_users: usize,
    pub total_events: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserDetailPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub user: Option<crate::types::UserDetail>,
    pub gamification: Option<crate::types::UserGamificationProfile>,
    pub enriched_achievements: Vec<EnrichedAchievementView>,
    pub achievements_count: usize,
    pub not_found: bool,
}
