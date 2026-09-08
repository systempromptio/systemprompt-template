//! Groups: the people container the directory maps AD groups onto, and the
//! subject marketplace entitlement is bound to.
//!
//! A group's membership has two sources that never overwrite each other —
//! `adfs` rows are replaced wholesale at every sign-in, `manual` rows are
//! what an admin added by hand — so a member row carries the set of sources
//! that put it there rather than a single one.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub member_count: i64,
    pub project_count: i64,
    pub active_members_30d: i64,
    pub requests_30d: i64,
    pub cost_30d_microdollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMemberRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub sources: Vec<String>,
    pub source_ad_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupAdMappingRow {
    pub ad_group: String,
    pub group_id: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGroupRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddGroupMemberRequest {
    pub user_id: UserId,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddAdMappingRequest {
    pub ad_group: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetGroupMarketplacesRequest {
    pub marketplace_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GroupUsageSummary {
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub active_users: i64,
}

/// Per-group user counts and activity, for the access-control overview.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupStats {
    pub group_id: String,
    pub user_count: i64,
    pub active_count: i64,
    pub total_events: i64,
    pub active_24h: i64,
}
