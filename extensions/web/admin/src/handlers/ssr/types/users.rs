use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UserMarketplaceRef {
    pub id: String,
    pub name: String,
    /// "default" (granted via YAML baseline) or "override" (granted via `access_control_rules`).
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepartmentGroup {
    pub department: String,
    pub users: Vec<EnrichedUserView>,
    pub user_count: usize,
    pub total_tokens: i64,
    pub total_sessions: i64,
}

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
    #[serde(default)]
    pub department: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub marketplaces: Vec<UserMarketplaceRef>,
    #[serde(default)]
    pub assigned_skills_count: i64,
    #[serde(default)]
    pub devices_count: i64,
    #[serde(default)]
    pub connected_agents: i64,
    #[serde(default)]
    pub total_agents: i64,
    #[serde(default)]
    pub lifetime_tokens: i64,
    /// "fresh" | "idle" | "stale" | "never"
    #[serde(default)]
    pub device_freshness: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsersPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub groups: Vec<DepartmentGroup>,
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
    #[serde(default)]
    pub departments: Vec<String>,
    pub runtime: Option<UserRuntimeView>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UserRuntimeView {
    pub connected_agents: i64,
    pub total_agents: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub last_bridge_version: Option<String>,
    pub last_os: Option<String>,
    pub last_hostname: Option<String>,
    pub last_heartbeat_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UserAssignmentSummary {
    pub skills_count: i64,
    pub marketplaces_count: i64,
    #[serde(default)]
    pub marketplaces: Vec<UserMarketplaceRef>,
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
