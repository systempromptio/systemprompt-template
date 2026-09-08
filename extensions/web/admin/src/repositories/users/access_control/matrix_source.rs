//! Mapping a resolver decision back to the band that produced it.
//!
//! `MatrixSource::layer` is the word the access-control UI colours a cell by,
//! so it has to name the band rather than the outcome: two `deny` cells decided
//! by a role rule and by a closed default are different facts to an operator
//! reading the matrix.

use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{DenyReason, MatchedBy};

use super::matrix::MatrixSource;

pub(super) fn allow_source(user_id: &UserId, matched_by: &MatchedBy) -> MatrixSource {
    match matched_by {
        MatchedBy::UserAllow => MatrixSource {
            layer: "user".into(),
            detail: format!("user:{user_id} allow"),
        },
        MatchedBy::RoleAllow { role } => MatrixSource {
            layer: "role".into(),
            detail: format!("role:{role} allow"),
        },
        MatchedBy::AttributeAllow { rule_type, value } => MatrixSource {
            layer: rule_type.to_string(),
            detail: format!("{rule_type}:{value} allow"),
        },
        MatchedBy::DefaultIncluded => MatrixSource {
            layer: "default".into(),
            detail: "default-included".into(),
        },
        MatchedBy::PolicyAllow { policy_id, detail } => MatrixSource {
            layer: "policy".into(),
            detail: format!("{policy_id}: {detail}"),
        },
    }
}

pub(super) fn deny_source(user_id: &UserId, reason: &DenyReason) -> MatrixSource {
    match reason {
        DenyReason::UserDeny { .. } => MatrixSource {
            layer: "user".into(),
            detail: format!("user:{user_id} deny"),
        },
        DenyReason::RoleDeny { role, .. } => MatrixSource {
            layer: "role".into(),
            detail: format!("role:{role} deny"),
        },
        DenyReason::AttributeDeny {
            rule_type, value, ..
        } => MatrixSource {
            layer: rule_type.to_string(),
            detail: format!("{rule_type}:{value} deny"),
        },
        // Why: everything else is the resolver closing the default rather than
        // a rule firing, so the cell reports the default layer and lets the
        // reason speak for itself.
        other => MatrixSource {
            layer: "default".into(),
            detail: other.to_string(),
        },
    }
}
