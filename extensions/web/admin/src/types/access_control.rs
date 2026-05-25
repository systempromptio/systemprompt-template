use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgTypeInfo, Decode, Encode, FromRow, Postgres, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    Role,
    Department,
    User,
}

impl fmt::Display for RuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Role => write!(f, "role"),
            Self::Department => write!(f, "department"),
            Self::User => write!(f, "user"),
        }
    }
}

impl TryFrom<String> for RuleType {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "role" => Ok(Self::Role),
            "department" => Ok(Self::Department),
            "user" => Ok(Self::User),
            other => Err(format!("invalid rule type: {other}")),
        }
    }
}

impl Type<Postgres> for RuleType {
    fn type_info() -> PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }
    fn compatible(ty: &PgTypeInfo) -> bool {
        <String as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for RuleType {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as Decode<Postgres>>::decode(value)?;
        Self::try_from(s).map_err(Into::into)
    }
}

impl<'q> Encode<'q, Postgres> for RuleType {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as Encode<Postgres>>::encode_by_ref(
            &match self {
                Self::Role => "role",
                Self::Department => "department",
                Self::User => "user",
            },
            buf,
        )
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

impl Type<Postgres> for AccessDecision {
    fn type_info() -> PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }
    fn compatible(ty: &PgTypeInfo) -> bool {
        <String as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for AccessDecision {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as Decode<Postgres>>::decode(value)?;
        Self::try_from(s).map_err(Into::into)
    }
}

impl<'q> Encode<'q, Postgres> for AccessDecision {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as Encode<Postgres>>::encode_by_ref(
            &match self {
                Self::Allow => "allow",
                Self::Deny => "deny",
            },
            buf,
        )
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessControlRuleInput {
    pub rule_type: RuleType,
    pub rule_value: String,
    pub access: AccessDecision,
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
