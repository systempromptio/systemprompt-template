//! `types::governance_labels` — how a governance decision names itself.
//!
//! Both functions replace derivations that had failed silently on the live
//! console: a stage column that was an em-dash on every row because it knew
//! only four policy names, and a "Tool" column showing a prompt sentinel and a
//! gateway-route id, neither of which was a tool. Nothing could assert on
//! either while they lived inside the page's private view module, which is the
//! reason they went unnoticed; these tests are the point of moving them.

use systemprompt_web_admin::types::governance_labels::{plane_of, target_label};

// Why: every policy this instance's `governance_decisions` table has ever held,
// read off the live database. If a producer starts writing a new one, the last
// assertion here is what says the page will still name it.
const OBSERVED_POLICIES: &[&str] = &[
    "agent_scope",
    "authentication",
    "authz",
    "authz_hook_fault",
    "authz_rule_based",
    "default_allow",
    "governance_allow",
    "governance_disabled",
    "quota",
    "rate_limit",
    "secret_scan",
    "tool_blocklist",
];

#[test]
fn every_policy_the_instance_writes_has_a_plane() {
    for policy in OBSERVED_POLICIES {
        let plane = plane_of(policy);
        assert!(
            plane.contains('·'),
            "{policy} fell through to the echo arm; it needs a plane"
        );
        assert!(
            !plane.contains('\u{2014}'),
            "{policy} rendered an em-dash, the exact failure this replaced"
        );
    }
}

#[test]
fn the_three_planes_are_distinguishable() {
    assert!(plane_of("secret_scan").starts_with("chain"));
    assert!(plane_of("default_allow").starts_with("chain"));
    assert!(plane_of("quota").starts_with("gateway"));
    assert!(plane_of("authentication").starts_with("gateway"));
    assert!(plane_of("authz_rule_based").starts_with("authz"));
}

#[test]
fn the_chain_stages_keep_their_evaluation_order() {
    assert!(plane_of("agent_scope").contains("1 scope"));
    assert!(plane_of("secret_scan").contains("2 secret"));
    assert!(plane_of("tool_blocklist").contains("3 blocklist"));
    assert!(plane_of("rate_limit").contains("4 rate"));
}

// Why: the arm that matters most. An unknown policy is not an error to swallow;
// it is a producer nobody knew about, and the page has to say its name.
#[test]
fn an_unknown_policy_names_itself_rather_than_disappearing() {
    assert_eq!(plane_of("some_future_policy"), "some_future_policy");
    assert_eq!(plane_of("unknown"), "unknown");
}

#[test]
fn a_prompt_is_named_a_prompt_and_not_a_tool() {
    assert_eq!(target_label("user_prompt", None), "prompt");
}

#[test]
fn a_real_tool_is_named_as_one() {
    assert_eq!(target_label("Bash", None), "tool Bash");
    assert_eq!(
        target_label("mcp__atlassian__listConfluenceContent", None),
        "tool mcp__atlassian__listConfluenceContent"
    );
}

// Why: the authz producers put an entity id in the tool column and the kind in
// `evaluated_rules`, so an id like `claude-star-4203d1` is only legible once
// the two are put back together.
#[test]
fn an_authorization_entity_is_named_by_its_kind() {
    assert_eq!(
        target_label("claude-star-4203d1", Some("gateway_route")),
        "gateway_route claude-star-4203d1"
    );
    assert_eq!(
        target_label("systemprompt", Some("mcp_server")),
        "mcp_server systemprompt"
    );
}

#[test]
fn an_empty_entity_type_is_treated_as_absent() {
    assert_eq!(target_label("Bash", Some("")), "tool Bash");
}

#[test]
fn an_empty_target_is_a_dash_rather_than_the_word_tool() {
    assert_eq!(target_label("", None), "\u{2014}");
}
