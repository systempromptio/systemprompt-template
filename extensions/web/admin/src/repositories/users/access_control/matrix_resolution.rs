//! Resolve one matrix cell, including its configured parent chains.
use super::matrix::MatrixSource;
use super::matrix_source::{allow_source, deny_source};
use crate::types::access_control::{AccessControlRule, AccessDecision};
use std::str::FromStr;
use systemprompt::identifiers::{RuleId, UserId};
use systemprompt_security::authz::{
    Access, AccessRule, Decision, EntityKind, SubjectAttributes, SubjectDimension,
};
fn as_access_rule(row: &AccessControlRule) -> AccessRule {
    AccessRule {
        id: RuleId::new(row.id.clone()),
        rule_type: row.rule_type.clone(),
        rule_value: row.rule_value.clone(),
        access: match row.access {
            AccessDecision::Allow => Access::Allow,
            AccessDecision::Deny => Access::Deny,
        },
        justification: None,
    }
}

// Why: the cell names a *subject*, not a user — the group and role rows of the
// audience matrix resolve through the same resolver call as a person does, and
// only the id and the role list differ between them.
pub(crate) struct MatrixCell<'a> {
    pub all_rules: &'a [AccessControlRule],
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub subject_id: &'a UserId,
    pub subject_roles: &'a [String],
    pub attributes: &'a SubjectAttributes,
    pub dimensions: &'a [SubjectDimension],
    pub default_included: bool,
}

pub(super) fn resolve_effective_with(
    cell: &MatrixCell<'_>,
    chains: &systemprompt_security::authz::ParentChainIndex,
) -> (String, MatrixSource) {
    let Ok(kind) = EntityKind::from_str(cell.entity_type) else {
        return (
            if cell.default_included {
                "allow"
            } else {
                "deny"
            }
            .to_owned(),
            MatrixSource {
                layer: "default".into(),
                detail: format!("unknown entity type: {}", cell.entity_type),
            },
        );
    };
    let rules: Vec<AccessRule> = cell
        .all_rules
        .iter()
        .filter(|r| r.entity_type == cell.entity_type && r.entity_id == cell.entity_id)
        .map(as_access_rule)
        .collect();

    let uid = cell.subject_id;
    let decision = chains.resolve(
        kind,
        cell.entity_id,
        systemprompt_security::authz::ResolveBase {
            rules: &rules,
            user_id: uid,
            user_roles: cell.subject_roles,
            default_included: Some(cell.default_included),
            attributes: cell.attributes,
            dimensions: cell.dimensions,
        },
    );

    match decision {
        Decision::Allow { matched_by } => ("allow".to_owned(), allow_source(uid, &matched_by)),
        // Why: a warning is a reach, but not a clean one. The cell names it so
        // an operator reading the matrix under warn mode sees which cells are
        // only open because enforcement is currently off.
        Decision::Warn { reason } => (
            "warn".to_owned(),
            MatrixSource {
                layer: "warn".into(),
                detail: reason.to_string(),
            },
        ),
        Decision::Deny { reason } => ("deny".to_owned(), deny_source(uid, &reason)),
        // Why: a hold is neither reach nor refusal, and flattening it into
        // either would misreport the matrix. The cell names it.
        Decision::Pending { reason } => (
            "pending".to_owned(),
            MatrixSource {
                layer: "approval".into(),
                detail: reason.to_string(),
            },
        ),
    }
}
