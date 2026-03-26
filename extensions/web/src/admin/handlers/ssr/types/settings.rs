use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SettingsView {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub notify_daily_summary: bool,
    pub notify_achievements: bool,
    pub leaderboard_opt_in: bool,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageItemView {
    pub label: String,
    pub current: usize,
    pub limit_display: String,
    pub is_unlimited: bool,
    pub percentage: usize,
    pub is_at_limit: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub settings: SettingsView,
    pub user_email: String,
    pub user_id: String,
    pub username: String,
    pub tier_name: String,
    pub is_premium: bool,
    pub usage_items: Vec<UsageItemView>,
}
