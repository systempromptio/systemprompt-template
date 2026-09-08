//! Projects: what work is attributed to, and an ACL subject in their own
//! right.
//!
//! Deliberately the same shape as [`super::groups`] rather than a shared
//! generic: the two diverge where it matters — a project has no system row
//! and no derived membership, and only groups carry marketplace entitlement —
//! so one type parameterised over both would have to encode the differences
//! anyway.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub active_members_30d: i64,
    pub requests_30d: i64,
    pub cost_30d_microdollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectMemberRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub sources: Vec<String>,
    pub source_ad_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectAdMappingRow {
    pub ad_group: String,
    pub project_id: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProjectRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddProjectMemberRequest {
    pub user_id: UserId,
}
