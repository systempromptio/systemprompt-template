//! Current database-backed Salesforce attributes for authorization.
//!
//! The dimension's values are the Salesforce server ids the user holds a
//! linked Username for, so a link gate written at `rule_value = <server id>`
//! opens exactly that org's server and no other.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

const SALESFORCE_SLUG: &str = "salesforce";

// Why: above `group` (150) and below core's `ROLE` (200). A link rule
// must out-rank the role band that grants the marketplace to every user, while
// remaining a narrower scope than a role.
const SALESFORCE_PRECEDENCE: u16 = 170;

#[must_use]
pub fn salesforce_rule_type() -> RuleType {
    RuleType::extension(SALESFORCE_SLUG)
        .unwrap_or_else(|e| unreachable!("`{SALESFORCE_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn salesforce_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: salesforce_rule_type(),
        label: "Salesforce link",
        precedence: SALESFORCE_PRECEDENCE,
    }
}

#[expect(
    clippy::unused_async,
    reason = "Retains the public invalidation API after removing membership caches"
)]
pub async fn invalidate(_user_id: &UserId) {}

#[derive(Debug)]
pub struct SalesforceAttributeProvider {
    pool: Arc<PgPool>,
}

impl SalesforceAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for SalesforceAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        salesforce_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(
            crate::repositories::users::salesforce_identity::list_linked_providers(
                &self.pool, user_id,
            )
            .await?,
        )
    }
}
