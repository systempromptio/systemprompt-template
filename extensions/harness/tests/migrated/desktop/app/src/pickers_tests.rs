//! Searching a picker's options.

use systemprompt_harness_views::pickers::search::matches;

#[test]
fn a_model_search_ignores_case_and_matches_any_field() {
    let fields = ["Claude Sonnet", "anthropic", "claude-sonnet-4"];
    assert!(matches("SONNET", &fields));
    assert!(matches("anthropic sonnet", &fields));
    assert!(!matches("gemini", &fields));
}

#[test]
#[ignore = "parity ledger: picker search does not ignore spaces and punctuation, so a spaced or hyphenated query misses a compact name"]
fn a_model_search_ignores_spaces_and_punctuation() {
    let fields = ["GPT-4o", "gpt4o"];
    assert!(matches("gpt 4 o", &fields));
    assert!(matches("gpt-4-o", &fields));
}
