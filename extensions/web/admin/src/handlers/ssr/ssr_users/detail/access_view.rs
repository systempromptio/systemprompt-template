//! Resolved resource rules and the read-only access overview.

use super::context::{AccessTabView, UserAccessRowView, UserAccessSectionView};
use super::load::AccessData;
use crate::types::access_control::{AccessControlRule, AccessDecision};
use systemprompt::identifiers::UserId;

pub(super) fn access_tab(mut data: AccessData, user_id: &UserId) -> AccessTabView {
    data.overview.set_permissions(data.matrix.as_ref());
    let Some(matrix) = data.matrix else {
        return AccessTabView {
            available: false,
            rules_available: false,
            overview: data.overview,
            has_groups: false,
            sections: Vec::new(),
        };
    };
    let sections = matrix
        .sections
        .into_iter()
        .map(|section| {
            let rows: Vec<UserAccessRowView> = section
                .rows
                .into_iter()
                .map(|row| {
                    let own = own_rule(&data.rules, &section.entity_type, &row.entity_id, user_id);
                    UserAccessRowView {
                        entity_type: section.entity_type.clone(),
                        effective_tone: effective_tone(&row.effective),
                        entity_id: row.entity_id,
                        entity_name: row.entity_name,
                        effective: row.effective,
                        layer: row.source.layer,
                        detail: row.source.detail,
                        state: own.map_or("inherit", |r| match r.access {
                            AccessDecision::Allow => "allow",
                            AccessDecision::Deny => "deny",
                        }),
                        rule_id: own.map(|r| r.id.clone()).unwrap_or_default(),
                    }
                })
                .collect();
            UserAccessSectionView {
                has_rows: !rows.is_empty(),
                rows,
                entity_type: section.entity_type,
                label: section.label,
            }
        })
        .collect();
    AccessTabView {
        available: true,
        rules_available: data.rules_available,
        overview: data.overview,
        has_groups: !matrix.user.group_ids.is_empty(),
        sections,
    }
}

// Why: `warn` and `pending` are real resolver outcomes, not failures. A warn
// is a reach that enforcement is currently letting through; a pending is a
// hold awaiting approval. Both get a tone that says "look here", not red.
const fn effective_tone(effective: &str) -> &'static str {
    match effective.as_bytes() {
        b"allow" => "ok",
        b"deny" => "err",
        b"warn" => "warn",
        _ => "info",
    }
}

fn own_rule<'a>(
    rules: &'a [AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    user_id: &UserId,
) -> Option<&'a AccessControlRule> {
    rules.iter().find(|r| {
        r.entity_type == entity_type
            && r.entity_id == entity_id
            && r.rule_type.as_str() == "user"
            && r.rule_value == user_id.as_str()
    })
}
