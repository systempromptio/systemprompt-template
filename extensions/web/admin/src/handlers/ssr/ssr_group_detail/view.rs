//! The group Access tab: effective grant plus the group's own toggle state.
//!
//! Everything else on this page is shaped by `people_view`, which the project
//! pages share.

use crate::types::access_control::{AccessControlRule, AccessDecision};
use crate::types::groups::GroupMemberRow;

use super::super::people_view::{MemberContext, MemberInput, member_rows};
use super::super::types::{AccessRowView, AccessSectionView, MemberRowView};
use super::data::MembersData;

pub(super) fn group_member_rows(data: &MembersData, can_manage: bool) -> Vec<MemberRowView> {
    let inputs: Vec<MemberInput<'_>> = data.rows.iter().map(as_member_input).collect();
    member_rows(
        &inputs,
        &MemberContext {
            usage: &data.usage,
            active: &data.active,
            can_manage,
        },
    )
}

fn as_member_input(row: &GroupMemberRow) -> MemberInput<'_> {
    MemberInput {
        user_id: row.user_id.as_str(),
        display_name: row.display_name.as_deref(),
        email: row.email.as_deref(),
        sources: &row.sources,
        source_ad_groups: &row.source_ad_groups,
    }
}

// Why: A cell's toggle state: `allow` or `deny` when this group's own band
// holds a rule, `inherit` when it does not and the effective value came from a
// wider band. Clearing a rule returns the cell to `inherit`, which is why the
// editor needs all three states and not a checkbox.
pub(super) fn access_sections(
    resolved: Vec<crate::repositories::users::access_control::MatrixSection>,
    rules: &[AccessControlRule],
    group_id: &str,
) -> Vec<AccessSectionView> {
    resolved
        .into_iter()
        .map(|section| {
            let rows = section
                .rows
                .into_iter()
                .map(|row| AccessRowView {
                    state: own_rule_state(rules, &section.entity_type, &row.entity_id, group_id),
                    entity_type: section.entity_type.clone(),
                    entity_id: row.entity_id,
                    entity_name: row.entity_name,
                    effective: row.effective,
                    layer: row.source.layer,
                    detail: row.source.detail,
                })
                .collect();
            AccessSectionView {
                rows,
                entity_type: section.entity_type,
                label: section.label,
            }
        })
        .collect()
}

fn own_rule_state(
    rules: &[AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    group_id: &str,
) -> &'static str {
    rules
        .iter()
        .find(|r| {
            r.entity_type == entity_type
                && r.entity_id == entity_id
                && r.rule_type.as_str() == "group"
                && r.rule_value == group_id
        })
        .map_or("inherit", |r| match r.access {
            AccessDecision::Allow => "allow",
            AccessDecision::Deny => "deny",
        })
}
