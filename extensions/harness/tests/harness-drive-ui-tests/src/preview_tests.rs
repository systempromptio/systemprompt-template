use systemprompt_harness_views::drive::model::{EXCERPT_CHARS, PREVIEW_LIMIT, PreviewText};
use systemprompt_identifiers::DocumentRevisionId;
use systemprompt_models::documents::client::DriveOutline;

fn outline(lines: Vec<String>) -> DriveOutline {
    DriveOutline {
        revision_id: DocumentRevisionId::new("11111111-2222-4333-8444-555555555555"),
        format: "markdown".to_owned(),
        lines,
    }
}

#[test]
fn previews_join_nodes_and_offer_anchors() {
    let preview = PreviewText::from_outline(&outline(vec![
        "n1:aaaa| # Title".to_owned(),
        "n2:bbbb| ".to_owned(),
        "n3:cccc| Body text".to_owned(),
        "garbage".to_owned(),
    ]));
    assert_eq!(preview.markdown, "# Title\n\nBody text");
    assert_eq!(
        preview
            .anchors
            .iter()
            .map(|a| a.node.as_str())
            .collect::<Vec<_>>(),
        vec!["n1", "n3"]
    );
    assert_eq!(preview.anchors[0].excerpt, "Title");
    assert!(!preview.truncated);
}

#[test]
fn long_previews_are_cut() {
    let line = format!("n1:aaaa| {}", "x".repeat(PREVIEW_LIMIT));
    let preview = PreviewText::from_outline(&outline(vec![line]));
    assert!(preview.truncated);
    assert_eq!(preview.markdown, "");
    assert_eq!(preview.anchors.len(), 1);
}

#[test]
fn long_paragraphs_give_a_short_excerpt() {
    let line = format!("n1:aaaa| {}", "y".repeat(EXCERPT_CHARS + 10));
    let preview = PreviewText::from_outline(&outline(vec![line]));
    let excerpt = &preview.anchors[0].excerpt;
    assert_eq!(excerpt.chars().count(), EXCERPT_CHARS + 1);
    assert!(excerpt.ends_with('…'));
}
