use systemprompt_harness_shared::code::fenced;

#[test]
fn fence_outgrows_backtick_runs_in_the_code() {
    assert_eq!(
        fenced("let a = 1;", Some("rust")),
        "```rust\nlet a = 1;\n```"
    );
    let tricky = "text ```` more";
    let block = fenced(tricky, None);
    assert!(block.starts_with("`````\n"));
    assert!(block.ends_with("\n`````"));
}

#[test]
fn fence_info_keeps_only_the_language_token() {
    assert!(fenced("x", Some("python title=a.py")).starts_with("```python\n"));
}

#[test]
fn trailing_newline_is_not_doubled() {
    assert_eq!(fenced("a\n", None), "```\na\n```");
}
