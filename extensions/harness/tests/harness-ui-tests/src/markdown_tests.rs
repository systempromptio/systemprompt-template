use std::ops::Range;

use harness_ui::markdown::{MarkdownBlock, MarkdownDocument};

fn ranges(document: &MarkdownDocument) -> Vec<Range<usize>> {
    document.blocks().iter().map(MarkdownBlock::range).collect()
}

fn pieces(text: &str) -> Vec<&str> {
    ranges(&MarkdownDocument::from_text(text))
        .into_iter()
        .map(|range| &text[range])
        .collect()
}

fn block_count(text: &str) -> usize {
    MarkdownDocument::from_text(text).blocks().len()
}

#[test]
fn paragraphs_split_on_blank_lines() {
    let text = "# Title\n\nFirst paragraph\ncontinues.\n\nSecond.";
    assert_eq!(
        pieces(text),
        vec!["# Title\n\n", "First paragraph\ncontinues.\n\n", "Second."]
    );
}

#[test]
fn fenced_code_keeps_its_blank_lines() {
    let text = "Intro\n\n```rust\nfn a() {}\n\nfn b() {}\n```\n\nAfter";
    let blocks = pieces(text);
    assert_eq!(blocks.len(), 3);
    assert!(blocks[1].contains("fn a() {}\n\nfn b() {}"));
    assert_eq!(blocks[2], "After");
}

#[test]
fn longer_fences_need_a_matching_close() {
    let text = "````\n```\n\ninner\n````\n\nout";
    assert_eq!(pieces(text).len(), 2);
}

#[test]
fn unclosed_fence_runs_to_the_end() {
    let text = "Text\n\n~~~\nstill code\n\nmore code";
    let blocks = pieces(text);
    assert_eq!(blocks.len(), 2);
    assert!(blocks[1].ends_with("more code"));
}

#[test]
fn indented_continuations_stay_with_their_block() {
    let text = "- item\n\n  continued paragraph\n\n- next";
    let blocks = pieces(text);
    assert_eq!(blocks.len(), 2);
    assert!(blocks[0].contains("continued paragraph"));
}

#[test]
fn whitespace_only_text_has_no_blocks() {
    assert!(
        MarkdownDocument::from_text("  \n\n \t\n")
            .blocks()
            .is_empty()
    );
    assert!(MarkdownDocument::from_text("").blocks().is_empty());
}

#[test]
fn appended_blocks_keep_document_offsets() {
    let mut document = MarkdownDocument::from_text("a\n\nb");
    assert_eq!(ranges(&document), vec![0..3, 3..4]);
    document.push_str("\n\nc");
    assert_eq!(ranges(&document), vec![0..3, 3..6, 6..7]);
}

#[test]
fn reference_definitions_keep_the_text_in_one_block() {
    assert_eq!(block_count("See [docs].\n\n[docs]: https://example.com"), 1);
    assert_eq!(block_count("Note[^1]\n\n[^1]: footnote"), 1);
    assert_eq!(
        block_count("See [docs].\n\nMore.\n\n[docs]: https://example.com"),
        1
    );
}

#[test]
fn inline_links_and_indented_labels_are_not_definitions() {
    assert_eq!(
        block_count("An [inline](https://example.com) link\n\nnext"),
        2
    );
    assert_eq!(
        block_count("para\n\n    [code]: not a definition\n\nend"),
        2
    );
}

const SAMPLE: &str = "# Plan\n\nWe will:\n\n1. read\n2. write\n\n```python\nprint(1)\n\nprint(2)\n```\n\n> quoted\n> more\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n- [ ] task\n- [x] done\n\n---\n\nLast line with `code`.";

#[test]
fn streaming_matches_a_full_split_at_every_step() {
    let mut document = MarkdownDocument::new();
    let mut boundary = 0;
    for (index, _) in SAMPLE.char_indices().skip(1).chain([(SAMPLE.len(), ' ')]) {
        document.push_str(&SAMPLE[boundary..index]);
        boundary = index;
        let expected = ranges(&MarkdownDocument::from_text(&SAMPLE[..index]));
        assert_eq!(ranges(&document), expected, "after {index} bytes");
    }
    assert_eq!(document.text(), SAMPLE);
}

#[test]
fn settled_blocks_are_not_rebuilt_when_text_grows() {
    let mut document = MarkdownDocument::from_text("First.\n\nSecond");
    let first = document.blocks()[0].text().as_ptr();
    let first_key = document.blocks()[0].key();
    document.push_str(" grows\n\nThird");
    assert_eq!(document.blocks().len(), 3);
    assert_eq!(document.blocks()[0].text().as_ptr(), first);
    assert_eq!(document.blocks()[0].key(), first_key);
    assert!(document.blocks()[2].is_tail());
    assert!(!document.blocks()[1].is_tail());
}

#[test]
fn tail_key_is_stable_while_the_tail_grows() {
    let mut document = MarkdownDocument::from_text("Intro.\n\nStreaming");
    let key = document.blocks()[1].key();
    document.push_str(" more words");
    assert_eq!(document.blocks()[1].key(), key);
}

#[test]
fn settled_keys_follow_content_not_position() {
    let a = MarkdownDocument::from_text("Same.\n\nOther\n\nend");
    let b = MarkdownDocument::from_text("Prefix.\n\nSame.\n\nOther\n\nend");
    assert_eq!(a.blocks()[0].key(), b.blocks()[1].key());
}

#[test]
fn duplicate_blocks_get_distinct_keys() {
    let document = MarkdownDocument::from_text("---\n\n---\n\nend");
    let keys: Vec<u64> = document.blocks().iter().map(MarkdownBlock::key).collect();
    assert_eq!(keys.len(), 3);
    assert_ne!(keys[0], keys[1]);
    assert_eq!(
        document.blocks()[0].content_hash(),
        document.blocks()[1].content_hash()
    );
}

#[test]
fn replacing_the_text_resplits_everything() {
    let mut document = MarkdownDocument::from_text("One.\n\nTwo.");
    assert!(document.set_text("Different.\n\nText."));
    assert_eq!(document.blocks()[0].text().as_ref(), "Different.");
    assert!(!document.set_text("Different.\n\nText."));
}
