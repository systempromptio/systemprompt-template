//! Access rule value types and their Postgres enum encodings.
//!
//! `rule_type` is core's [`RuleType`][systemprompt_security::authz::RuleType],
//! re-exported here so the admin CRUD surface and the resolver cannot drift
//! apart: this crate used to keep a parallel `Role | Department | User` enum,
//! and a `department` row that the matrix rendered was invisible to the
//! resolver as a direct result. Core owns `user` and `role`; `department` is
//! minted by [`crate::authz::department`].

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgTypeInfo;
use sqlx::{Decode, Encode, FromRow, Postgres, Type};
pub use systemprompt_security::authz::RuleType;

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

impl Encode<'_, Postgres> for AccessDecision {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer,
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
    // Why: polymorphic entity reference (role/department/user), no single typed-ID equivalent
    pub entity_id: String,
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
    // Why: polymorphic entity reference (role/department/user), no single typed-ID equivalent
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
    // Why: polymorphic entity reference (role/department/user), no single typed-ID equivalent
    pub entity_id: Option<String>,
}
