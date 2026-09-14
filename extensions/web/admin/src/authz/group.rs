//! Current database-backed group attributes for authorization.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

const GROUP_SLUG: &str = "group";

// Why: below core's `ROLE` (200) so a group rule out-ranks the role band that
// grants a marketplace to every user, and above `project` (140) so a project
// rule stays the narrower of the two.
const GROUP_PRECEDENCE: u16 = 150;

#[must_use]
pub fn group_rule_type() -> RuleType {
    RuleType::extension(GROUP_SLUG)
        .unwrap_or_else(|e| unreachable!("`{GROUP_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn group_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: group_rule_type(),
        label: "Group",
        precedence: GROUP_PRECEDENCE,
    }
}

#[expect(
    clippy::unused_async,
    reason = "Retains the public invalidation API after removing membership caches"
)]
pub async fn invalidate(_user_id: &UserId) {}

#[derive(Debug)]
pub struct GroupAttributeProvider {
    pool: Arc<PgPool>,
}

impl GroupAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for GroupAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        group_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(
            crate::repositories::groups::members::list_group_ids_for_user(&self.pool, user_id)
                .await?
                .into_iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
        )
    }
}
