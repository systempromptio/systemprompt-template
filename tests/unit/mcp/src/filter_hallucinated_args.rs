//! `filter_hallucinated_args` drops the output-format flags models routinely
//! append to `systemprompt` invocations, while leaving every real argument
//! (including their values and positional args) untouched.

use systemprompt_mcp_agent::filter_hallucinated_args;

fn v(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn empty_input_stays_empty() {
    let out = filter_hallucinated_args(Vec::new());
    assert!(out.is_empty());
}

#[test]
fn passes_through_when_no_hallucinated_flags() {
    let input = v(&["core", "skills", "list", "--limit", "10"]);
    let out = filter_hallucinated_args(input.clone());
    assert_eq!(out, input);
}

#[test]
fn strips_each_known_hallucinated_flag() {
    for flag in ["--json", "--output-format", "--format"] {
        let out = filter_hallucinated_args(v(&["core", "skills", "list", flag]));
        assert_eq!(
            out,
            v(&["core", "skills", "list"]),
            "flag {flag} should have been stripped"
        );
    }
}

#[test]
fn strips_multiple_and_preserves_order_of_survivors() {
    let out = filter_hallucinated_args(v(&[
        "--json", "core", "--format", "skills", "list", "--output-format",
    ]));
    assert_eq!(out, v(&["core", "skills", "list"]));
}

#[test]
fn removes_every_occurrence_of_a_repeated_flag() {
    let out = filter_hallucinated_args(v(&["--json", "core", "--json", "list"]));
    assert_eq!(out, v(&["core", "list"]));
}

#[test]
fn only_matches_exact_flag_tokens_not_substrings_or_values() {
    // A `--format`-shaped value or an unrelated flag that merely contains the
    // text must survive: the filter matches whole argv tokens only.
    let input = v(&["run", "--output", "json", "--formatter", "--json-lines"]);
    let out = filter_hallucinated_args(input.clone());
    assert_eq!(out, input);
}
