//! The installation-wide subject: every user holds the one value `default`.
//!
//! A quota window keyed on `subject: organization` needs a provider to name
//! the bucket it counts into, and under `quota_fault_mode: closed` a window
//! with no provider refuses every request. This is that provider. The value
//! is a constant because this installation is one tenant; a multi-org estate
//! would read it off the user's row instead.

use async_trait::async_trait;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

const ORGANIZATION_SLUG: &str = "organization";

pub const ORGANIZATION_DEFAULT: &str = "default";

// Why: the widest band there is — everyone holds it — so it sits above
// core's `ROLE` (200). A rule written against it is a rule about the whole
// installation and must lose to every narrower statement.
const ORGANIZATION_PRECEDENCE: u16 = 300;

#[must_use]
pub fn organization_rule_type() -> RuleType {
    RuleType::extension(ORGANIZATION_SLUG)
        .unwrap_or_else(|e| unreachable!("`{ORGANIZATION_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn organization_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: organization_rule_type(),
        label: "Organization",
        precedence: ORGANIZATION_PRECEDENCE,
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OrganizationAttributeProvider;

#[async_trait]
impl SubjectAttributeProvider for OrganizationAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        organization_dimension()
    }

    async fn values_for(&self, _user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(vec![ORGANIZATION_DEFAULT.to_owned()])
    }
}
