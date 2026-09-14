//! The gateway transcript-framing stripper.
//!
//! A `/v1/messages` turn is stored with `=== USER PROMPT ===` /
//! `=== ASSISTANT ANSWER ===` headers around it. Every user-facing surface
//! cuts that framing, and the SQL preview CTE cuts the same thing in Postgres,
//! so the rule is pinned here rather than left to two implementations to agree
//! on by accident.

use systemprompt_web_admin::repositories::analytics::conversations::strip_gateway_markers;

#[test]
fn the_user_prompt_header_is_removed() {
    assert_eq!(
        strip_gateway_markers("=== USER PROMPT ===\nwhat is the deploy command?"),
        "what is the deploy command?"
    );
}

#[test]
fn the_assistant_half_is_cut_at_its_marker() {
    let stored = "=== USER PROMPT ===\nhello\n=== ASSISTANT ANSWER ===\ndata: {\"type\":\"x\"}";
    assert_eq!(strip_gateway_markers(stored), "hello");
}

#[test]
fn a_body_with_no_markers_is_returned_trimmed_and_otherwise_intact() {
    assert_eq!(
        strip_gateway_markers("  plain prompt with === inside  "),
        "plain prompt with === inside"
    );
}

#[test]
fn every_repeat_of_the_user_marker_is_removed_not_only_the_first() {
    assert_eq!(
        strip_gateway_markers("=== USER PROMPT ===a=== USER PROMPT ===b"),
        "ab"
    );
}

#[test]
fn an_empty_body_stays_empty_rather_than_becoming_a_marker_fragment() {
    assert_eq!(strip_gateway_markers("=== USER PROMPT ==="), "");
}
