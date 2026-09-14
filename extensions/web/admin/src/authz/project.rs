//! Current database-backed project attributes for authorization.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

const PROJECT_SLUG: &str = "project";

// Why: the narrowest of the tenant dimensions — a project rule is the most
// specific statement an operator can make about who a user is, so it out-ranks
// their group (150) and every band above it.
const PROJECT_PRECEDENCE: u16 = 140;

#[must_use]
pub fn project_rule_type() -> RuleType {
    RuleType::extension(PROJECT_SLUG)
        .unwrap_or_else(|e| unreachable!("`{PROJECT_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn project_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: project_rule_type(),
        label: "Project",
        precedence: PROJECT_PRECEDENCE,
    }
}

#[expect(
    clippy::unused_async,
    reason = "Retains the public invalidation API after removing membership caches"
)]
pub async fn invalidate(_user_id: &UserId) {}

#[derive(Debug)]
pub struct ProjectAttributeProvider {
    pool: Arc<PgPool>,
}

impl ProjectAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for ProjectAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        project_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(
            crate::repositories::projects::members::list_project_ids_for_user(&self.pool, user_id)
                .await?
                .into_iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
        )
    }
}
