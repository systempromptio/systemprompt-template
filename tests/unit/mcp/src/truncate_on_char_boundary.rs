//! `truncate_on_char_boundary` caps a string at `max_bytes`, snapping the cut
//! down to a UTF-8 boundary and appending "..." only when it actually
//! truncated. The rejection-audit path relies on it never splitting a
//! multi-byte codepoint (which would panic on the byte slice).

use systemprompt_mcp_shared::{MAX_REASON_LEN, truncate_on_char_boundary};

#[test]
fn shorter_than_max_is_returned_verbatim() {
    let out = truncate_on_char_boundary("hi", 10);
    assert_eq!(out, "hi");
    assert!(!out.ends_with("..."));
}

#[test]
fn exactly_max_is_not_truncated() {
    // len == max_bytes takes the `<=` fast path: no cut, no suffix.
    let out = truncate_on_char_boundary("abcde", 5);
    assert_eq!(out, "abcde");
    assert!(!out.ends_with("..."));
}

#[test]
fn longer_ascii_is_cut_and_suffixed() {
    let out = truncate_on_char_boundary("abcdef", 5);
    assert_eq!(out, "abcde...");
}

#[test]
fn cut_snaps_down_past_a_multibyte_boundary() {
    // "aaé" is a,a,(0xC3 0xA9) = 4 bytes. max_bytes=3 lands inside 'é', so the
    // cut must retreat to byte 2 — "aa" — never emitting a split codepoint.
    let input = "aaé";
    assert_eq!(input.len(), 4);
    let out = truncate_on_char_boundary(input, 3);
    assert_eq!(out, "aa...");
    // Round-trips as valid UTF-8 (String guarantees it; assert intent anyway).
    assert!(std::str::from_utf8(out.as_bytes()).is_ok());
}

#[test]
fn multibyte_exactly_at_max_is_kept_whole() {
    // "aé" is 3 bytes; max_bytes=3 == len, so it is returned intact, proving
    // the boundary walk is only reached on the truncation path.
    let input = "aé";
    assert_eq!(input.len(), 3);
    let out = truncate_on_char_boundary(input, 3);
    assert_eq!(out, "aé");
}

#[test]
fn multibyte_string_longer_than_max_never_splits() {
    // A run of 'é' (2 bytes each); cutting at an odd max lands mid-codepoint
    // every time, so the result's pre-suffix body must be an even byte length.
    let input = "é".repeat(10); // 20 bytes
    let out = truncate_on_char_boundary(&input, 5);
    let body = out.strip_suffix("...").expect("was truncated, so suffixed");
    assert_eq!(body, "éé"); // 4 bytes — snapped down from 5
    assert!(body.len() <= 5);
}

#[test]
fn respects_the_max_reason_len_constant() {
    // The reason budget the rejection audit actually uses.
    let input = "x".repeat(MAX_REASON_LEN + 50);
    let out = truncate_on_char_boundary(&input, MAX_REASON_LEN);
    assert!(out.ends_with("..."));
    // Body is capped at MAX_REASON_LEN, so the whole string never exceeds
    // MAX_REASON_LEN + 3 (the "..." suffix), as the doc comment promises.
    assert_eq!(out.len(), MAX_REASON_LEN + 3);
}
