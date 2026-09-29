//! The user's effective role set, exposed as a quota subject.
//!
//! Core's authorization resolver already reads roles off the request and
//! ignores this dimension (`ladder` filters `RuleType::ROLE` out), so
//! registering it changes no access decision. What it adds is a provider the
//! gateway's quota resolver can find for `subject: role` windows, which
//! otherwise fault as "no subject attribute provider for this dimension".
//!
//! The values are the two halves `users::roles` draws: what an admin granted
//! by hand, then what the directory projected. A quota window counts into the
//! first value, so a manual grant — the operator's explicit decision about
//! this person — is the bucket they land in.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, ROLE_PRECEDENCE, RuleType, SubjectAttributeProvider, SubjectDimension,
};

use crate::repositories::users::roles::{list_directory_roles, list_manual_roles};

#[must_use]
pub const fn role_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: RuleType::ROLE,
        label: "Role",
        precedence: ROLE_PRECEDENCE,
    }
}

// Why: manual grants lead and a role held both ways is listed once, in the
// manual position — the same reading `users::roles` gives the role editor.
#[must_use]
pub fn effective_roles(manual: Vec<String>, directory: Vec<String>) -> Vec<String> {
    let mut roles = manual;
    for role in directory {
        if !roles.contains(&role) {
            roles.push(role);
        }
    }
    roles
}

#[derive(Debug)]
pub struct RoleAttributeProvider {
    pool: Arc<PgPool>,
}

impl RoleAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for RoleAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        role_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        let manual = list_manual_roles(&self.pool, user_id).await?;
        let directory = list_directory_roles(&self.pool, user_id).await?;
        Ok(effective_roles(manual, directory))
    }
}
