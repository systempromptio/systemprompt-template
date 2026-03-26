use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub keywords: Vec<String>,
    pub author_name: String,
    pub enabled: bool,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_server_count: usize,
    pub hook_count: usize,
    pub roles: Vec<String>,
    pub visibility_rules: Vec<VisibilityRule>,
    pub total_events: i64,
    pub unique_users: i64,
    pub active_users_7d: i64,
    pub active_users_30d: i64,
    pub avg_rating: f64,
    pub rating_count: i64,
    pub rank_score: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PluginUser {
    pub user_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub department: Option<String>,
    pub event_count: i64,
    pub last_used: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VisibilityRule {
    pub id: String,
    pub plugin_id: String,
    pub rule_type: String,
    pub rule_value: String,
    pub access: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PluginRating {
    pub id: String,
    pub plugin_id: String,
    pub user_id: String,
    pub rating: i16,
    pub review: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PluginUsageAggregate {
    pub plugin_id: String,
    pub total_events: i64,
    pub unique_users: i64,
    pub active_users_7d: i64,
    pub active_users_30d: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PluginRatingAggregate {
    pub plugin_id: String,
    pub avg_rating: f64,
    pub rating_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct SubmitRatingRequest {
    pub user_id: String,
    pub rating: i16,
    pub review: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVisibilityRequest {
    pub rules: Vec<VisibilityRuleInput>,
}

#[derive(Debug, Deserialize)]
pub struct VisibilityRuleInput {
    pub rule_type: String,
    pub rule_value: String,
    pub access: String,
}

#[derive(Debug, Deserialize)]
pub struct MarketplaceQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub sort: Option<String>,
}
