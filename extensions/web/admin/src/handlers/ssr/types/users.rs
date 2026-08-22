//! Template context types for the user pages.

use serde::Serialize;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserMarketplaceRef {
    pub id: String,
    pub name: String,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DepartmentGroup {
    pub department: String,
    pub users: Vec<EnrichedUserView>,
    pub user_count: usize,
    pub total_tokens: i64,
    pub total_sessions: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnrichedUserView {
    pub user_id: UserId,
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
    pub tokens_count: i64,
    #[serde(default)]
    pub lifetime_tokens: i64,
    #[serde(default)]
    pub token_freshness: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PageStatView {
    pub value: i64,
    pub label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UsersPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub groups: Vec<DepartmentGroup>,
    pub total_users: usize,
    pub active_users: usize,
    pub total_events: i64,
    #[serde(default)]
    pub page_stats: Vec<PageStatView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserDetailPageData {
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
    pub user_tokens: Vec<UserTokenView>,
    #[serde(default)]
    pub user_tokens_count: i64,
    #[serde(default)]
    pub departments: Vec<String>,
    pub runtime: Option<UserRuntimeView>,
    #[serde(default)]
    pub effective_permissions:
        Option<crate::repositories::governance::effective::EffectivePermissions>,
    #[serde(default)]
    pub has_effective_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub(crate) struct UserRuntimeView {
    pub requests: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub last_request_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct UserAssignmentSummary {
    pub skills_count: i64,
    pub marketplaces_count: i64,
    #[serde(default)]
    pub marketplaces: Vec<UserMarketplaceRef>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserTokenView {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked: bool,
}
