//! The subject dimensions the gateway's quota windows key on: the
//! installation-wide `organization`, the re-exposed `role`, and the ordering
//! that puts a person's attribution key first.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_security::authz::{ROLE_PRECEDENCE, RuleType};
use systemprompt_web_admin::authz::group::group_dimension;
use systemprompt_web_admin::authz::organization::{
    ORGANIZATION_DEFAULT, organization_dimension, organization_rule_type,
};
use systemprompt_web_admin::authz::primary::lead_with_primary;
use systemprompt_web_admin::authz::role::{effective_roles, role_dimension};

fn ids(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

#[test]
fn organization_is_the_widest_band_and_names_the_policy_subject() {
    let dimension = organization_dimension();
    assert_eq!(dimension.rule_type, organization_rule_type());
    assert_eq!(
        organization_rule_type().as_str(),
        "organization",
        "must equal the `subject:` in services/gateway/policies.yaml or the window faults"
    );
    assert!(
        dimension.precedence > ROLE_PRECEDENCE
            && dimension.precedence > group_dimension().precedence,
        "everyone holds it, so it must lose to every narrower statement"
    );
    assert_eq!(ORGANIZATION_DEFAULT, "default");
}

#[test]
fn role_re_exposes_core_dimension_unchanged() {
    let dimension = role_dimension();
    assert_eq!(dimension.rule_type, RuleType::ROLE);
    assert_eq!(dimension.precedence, ROLE_PRECEDENCE);
}

#[test]
fn manual_grants_lead_and_a_role_held_both_ways_is_listed_once() {
    let roles = effective_roles(ids(&["platform_admin"]), ids(&["platform_admin", "user"]));
    assert_eq!(roles, ids(&["platform_admin", "user"]));

    let directory_only = effective_roles(Vec::new(), ids(&["user"]));
    assert_eq!(directory_only, ids(&["user"]));
}

#[test]
fn the_primary_moves_to_the_front_and_nothing_else_changes() {
    let ordered = lead_with_primary(Some("grp-c"), ids(&["grp-a", "grp-b", "grp-c", "grp-d"]));
    assert_eq!(ordered, ids(&["grp-c", "grp-a", "grp-b", "grp-d"]));

    let already_first = lead_with_primary(Some("grp-a"), ids(&["grp-a", "grp-b"]));
    assert_eq!(already_first, ids(&["grp-a", "grp-b"]));
}

#[test]
fn a_primary_the_person_has_left_or_never_set_leaves_the_order_alone() {
    let stale = lead_with_primary(Some("grp-gone"), ids(&["grp-a", "grp-b"]));
    assert_eq!(
        stale,
        ids(&["grp-a", "grp-b"]),
        "a quota must not count into a container they left"
    );

    let unset = lead_with_primary(None, ids(&["grp-b", "grp-a"]));
    assert_eq!(unset, ids(&["grp-b", "grp-a"]));

    assert!(lead_with_primary(Some("grp-a"), Vec::new()).is_empty());
}
