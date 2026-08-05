//! `short_id` truncates by character count, not byte count: a multi-byte id
//! must not be sliced mid-codepoint, and the ellipsis appears only when
//! something was actually dropped.

use systemprompt_web_shared::format::short_id;

#[test]
fn ids_of_twelve_chars_or_fewer_are_returned_whole() {
    assert_eq!(short_id(""), "");
    assert_eq!(short_id("abc"), "abc");
    // Exactly at the boundary: 12 chars, still no ellipsis.
    assert_eq!(short_id("012345678901"), "012345678901");
}

#[test]
fn longer_ids_keep_the_first_twelve_chars_and_gain_an_ellipsis() {
    assert_eq!(short_id("0123456789012"), "012345678901\u{2026}");
    assert_eq!(short_id("trace_0123456789abcdef"), "trace_012345\u{2026}");
}

#[test]
fn truncation_counts_chars_not_bytes() {
    // 13 multi-byte chars: a byte-based slice would panic or split a codepoint.
    let id = "ααααααααααααα";
    let out = short_id(id);
    assert_eq!(out.chars().count(), 13, "12 kept chars plus the ellipsis");
    assert!(out.ends_with('\u{2026}'));
    assert_eq!(out.trim_end_matches('\u{2026}').chars().count(), 12);
}
