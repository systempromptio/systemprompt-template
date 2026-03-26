use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessControlRule {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub rule_type: String,
    pub rule_value: String,
    pub access: String,
    pub default_included: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessControlRuleInput {
    pub rule_type: String,
    pub rule_value: String,
    pub access: String,
    #[serde(default)]
    pub default_included: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntityRulesRequest {
    pub rules: Vec<AccessControlRuleInput>,
    #[serde(default)]
    pub sync_yaml: bool,
}

#[derive(Debug, Deserialize)]
pub struct BulkEntityRef {
    pub entity_type: String,
    pub entity_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkAssignRequest {
    pub entities: Vec<BulkEntityRef>,
    pub rules: Vec<AccessControlRuleInput>,
    #[serde(default)]
    pub sync_yaml: bool,
}

#[derive(Debug, Deserialize)]
pub struct AccessControlQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}
