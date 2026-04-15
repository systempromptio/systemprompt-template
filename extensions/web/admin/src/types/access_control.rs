use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    Role,
    Department,
}

impl fmt::Display for RuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Role => write!(f, "role"),
            Self::Department => write!(f, "department"),
        }
    }
}

impl TryFrom<String> for RuleType {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "role" => Ok(Self::Role),
            "department" => Ok(Self::Department),
            other => Err(format!("invalid rule type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessDecision {
    Allow,
    Deny,
}

impl fmt::Display for AccessDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Deny => write!(f, "deny"),
        }
    }
}

impl TryFrom<String> for AccessDecision {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            other => Err(format!("invalid access decision: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessControlRule {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    #[sqlx(try_from = "String")]
    pub rule_type: RuleType,
    pub rule_value: String,
    #[sqlx(try_from = "String")]
    pub access: AccessDecision,
    pub default_included: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessControlRuleInput {
    pub rule_type: RuleType,
    pub rule_value: String,
    pub access: AccessDecision,
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
