//! Streamed markdown blocks: an open code fence kept apart from settled text,
//! fences closed across deltas, and reference definitions kept with their
//! uses.

use systemprompt_harness_shared::markdown::MarkdownDocument;

fn texts(document: &MarkdownDocument) -> Vec<String> {
    document
        .blocks()
        .iter()
        .map(|block| block.text().to_string())
        .collect()
}

#[test]
fn an_unfinished_code_fence_is_split_from_the_settled_text() {
    let mut document = MarkdownDocument::new();
    document.push_str("Here is the fix.\n\n");
    document.push_str("```rust\nfn main() {\n\n    run();");
    let blocks = document.blocks();
    assert_eq!(blocks.len(), 2);
    assert!(!blocks[0].is_tail());
    assert!(blocks[1].is_tail());
    assert_eq!(blocks[0].text().as_ref(), "Here is the fix.");
    assert!(blocks[1].text().starts_with("```rust"));
}

#[test]
fn a_closed_code_fence_is_one_block_and_later_prose_starts_another() {
    let mut document = MarkdownDocument::new();
    document.push_str("```sh\nmake\n\nmake test\n");
    document.push_str("```\n\nAll green.");
    assert_eq!(
        texts(&document),
        vec![
            "```sh\nmake\n\nmake test\n```".to_owned(),
            "All green.".to_owned()
        ]
    );
}

#[test]
fn a_tilde_fence_split_across_deltas_keeps_its_blank_lines() {
    let mut document = MarkdownDocument::new();
    document.push_str("~~~\nfirst\n\n");
    document.push_str("second\n~");
    document.push_str("~~\n\nafter");
    assert_eq!(
        texts(&document),
        vec!["~~~\nfirst\n\nsecond\n~~~".to_owned(), "after".to_owned()]
    );
}

#[test]
fn reference_definitions_keep_the_text_in_one_block() {
    for text in [
        "See [the guide][g].\n\n[g]: https://example.com",
        "See [the guide][g].\n\n   [g]: https://example.com",
        "See [the guide][g].\n\n[g]:\n  https://example.com\n  \"Title\"",
    ] {
        assert_eq!(
            MarkdownDocument::from_text(text).blocks().len(),
            1,
            "{text}"
        );
    }
}
