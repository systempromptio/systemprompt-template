use serde::Serialize;

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
    #[serde(default)]
    pub department: String,
    #[serde(default)]
    pub assigned_skills_count: i64,
    #[serde(default)]
    pub assigned_marketplaces_count: i64,
    #[serde(default)]
    pub devices_count: i64,
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
    pub not_found: bool,
    #[serde(default)]
    pub user_department: String,
    #[serde(default)]
    pub user_assignments: UserAssignmentSummary,
    #[serde(default)]
    pub user_devices: Vec<UserDeviceView>,
    #[serde(default)]
    pub user_devices_count: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UserAssignmentSummary {
    pub skills_count: i64,
    pub marketplaces_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserDeviceView {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub platform: Option<String>,
    pub app_version: Option<String>,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked: bool,
}
