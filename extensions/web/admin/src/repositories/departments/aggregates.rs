//! Cross-user aggregates used by the user-management views: marketplace
//! overrides and per-user skill / device counts keyed by department.

use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DepartmentUserManagementAggregate {
    pub user_id: UserId,
    pub department: String,
    pub assigned_skills_count: i64,
    pub tokens_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DepartmentUserMarketplaceOverride {
    pub user_id: UserId,
    pub department: String,
    // Why: polymorphic entity reference (gateway_route/mcp_server), no single typed-ID equivalent
    pub entity_id: String,
    pub access: String,
}

// Why: A user receives overrides from rules matching either their own id or
// their department, so the same entity can appear under both scopes.
