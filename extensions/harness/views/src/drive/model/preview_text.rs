//! A document's preview text built from its outline nodes, with the anchors a
//! comment can attach to.

use systemprompt_models::documents::client::DriveOutline;

pub const PREVIEW_LIMIT: usize = 200_000;
pub const EXCERPT_CHARS: usize = 60;

/// A paragraph a new comment can be pinned to: its outline node and the start
/// of its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorChoice {
    pub node: String,
    pub excerpt: String,
}

/// The preview of a document: its text as Markdown up to [`PREVIEW_LIMIT`],
/// every paragraph a comment can attach to, and whether the text was cut.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreviewText {
    pub markdown: String,
    pub anchors: Vec<AnchorChoice>,
    pub truncated: bool,
}

impl PreviewText {
    pub fn from_outline(outline: &DriveOutline) -> Self {
        let mut preview = Self::default();
        for line in outline.nodes() {
            let text = line.text.trim_end();
            if text.trim().is_empty() {
                continue;
            }
            preview.anchors.push(AnchorChoice {
                node: line.node.to_owned(),
                excerpt: excerpt(text),
            });
            if preview.truncated {
                continue;
            }
            if preview.markdown.len() + text.len() + 2 > PREVIEW_LIMIT {
                preview.truncated = true;
                continue;
            }
            if !preview.markdown.is_empty() {
                preview.markdown.push_str("\n\n");
            }
            preview.markdown.push_str(text);
        }
        preview
    }
}

fn excerpt(text: &str) -> String {
    let plain = text.trim_start_matches(['#', '-', '*', '>', ' ', '\t']);
    let mut cut: String = plain.chars().take(EXCERPT_CHARS).collect();
    if plain.chars().count() > EXCERPT_CHARS {
        cut.push('…');
    }
    cut
}
