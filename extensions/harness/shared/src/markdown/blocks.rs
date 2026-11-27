//! Splitting markdown into top-level blocks, and a document that re-splits only
//! its tail as text streams in.
//!
//! A block boundary is a blank line followed by an unindented line, outside
//! fenced code. Joining the blocks renders the same as rendering the whole
//! text, with one deliberate exception handled up front: link reference
//! definitions resolve across the whole document, so text containing one is
//! kept as a single block.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash as _, Hasher as _};
use std::ops::Range;

use gpui_kit::SharedString;

const TAIL_SALT: u64 = 0x7a11_b10c_5eed_0001;

/// One top-level block of a [`MarkdownDocument`].
///
/// Its `key` is what the renderer caches the parse under: the content
/// hash (with an occurrence count) for settled blocks, the start offset for the
/// last block, which may still grow.
#[derive(Debug, Clone)]
pub struct MarkdownBlock {
    range: Range<usize>,
    hash: u64,
    key: u64,
    text: SharedString,
    tail: bool,
}

impl MarkdownBlock {
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    #[must_use]
    pub const fn content_hash(&self) -> u64 {
        self.hash
    }

    #[must_use]
    pub const fn key(&self) -> u64 {
        self.key
    }

    #[must_use]
    pub const fn text(&self) -> &SharedString {
        &self.text
    }

    #[must_use]
    pub const fn is_tail(&self) -> bool {
        self.tail
    }
}

/// Markdown text kept as blocks, so appending text re-splits and re-hashes only
/// the last block.
///
/// `push_str` appends streamed text; `set_text` replaces it, reusing every
/// block before the last when the new text keeps that prefix, and returns
/// whether anything changed.
#[derive(Debug, Clone, Default)]
pub struct MarkdownDocument {
    text: String,
    blocks: Vec<MarkdownBlock>,
    single: bool,
}

impl MarkdownDocument {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_text(text: &str) -> Self {
        let mut document = Self::new();
        document.set_text(text);
        document
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn blocks(&self) -> &[MarkdownBlock] {
        &self.blocks
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn push_str(&mut self, delta: &str) {
        if delta.is_empty() {
            return;
        }
        let mut next = String::with_capacity(self.text.len() + delta.len());
        next.push_str(&self.text);
        next.push_str(delta);
        self.set_text(&next);
    }

    pub fn set_text(&mut self, text: &str) -> bool {
        if text == self.text {
            return false;
        }
        let keep = self.reusable_prefix(text);
        let single = if keep == 0 {
            has_reference_definitions(text)
        } else {
            self.single || has_reference_definitions(&text[keep..])
        };

        if single || keep == 0 {
            self.blocks.clear();
            self.single = single;
            let ranges = if single {
                whole(text, 0)
            } else {
                split_blocks(text, 0)
            };
            self.push_ranges(text, ranges);
        } else {
            self.blocks.pop();
            let ranges = split_blocks(&text[keep..], keep);
            self.push_ranges(text, ranges);
        }
        self.text.clear();
        self.text.push_str(text);
        self.mark_tail();
        true
    }

    fn reusable_prefix(&self, text: &str) -> usize {
        let Some(last) = self.blocks.last() else {
            return 0;
        };
        let start = last.range.start;
        let shared = start <= text.len()
            && text.is_char_boundary(start)
            && text.as_bytes()[..start] == self.text.as_bytes()[..start];
        if shared { start } else { 0 }
    }

    fn push_ranges(&mut self, text: &str, ranges: Vec<Range<usize>>) {
        for range in ranges {
            let content = text[range.clone()].trim_end();
            self.blocks.push(MarkdownBlock {
                hash: hash_text(content),
                key: 0,
                text: SharedString::from(content.to_owned()),
                range,
                tail: false,
            });
        }
    }

    fn mark_tail(&mut self) {
        let count = self.blocks.len();
        let mut seen: HashMap<u64, u64> = HashMap::new();
        for (index, block) in self.blocks.iter_mut().enumerate() {
            let occurrence = seen.entry(block.hash).or_insert(0);
            block.tail = index + 1 == count;
            block.key = if block.tail {
                mix(TAIL_SALT, block.range.start as u64)
            } else {
                mix(block.hash, *occurrence)
            };
            *occurrence += 1;
        }
    }
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn mix(a: u64, b: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    b.hash(&mut hasher);
    hasher.finish()
}

fn whole(text: &str, base: usize) -> Vec<Range<usize>> {
    if text.trim().is_empty() {
        Vec::new()
    } else {
        std::iter::once(base..base + text.len()).collect()
    }
}

struct Fence {
    marker: u8,
    len: usize,
}

fn strip_indent(line: &str, max: usize) -> Option<&str> {
    let spaces = line.bytes().take_while(|byte| *byte == b' ').count();
    if spaces > max {
        None
    } else {
        Some(&line[spaces..])
    }
}

fn fence_open(line: &str) -> Option<Fence> {
    let rest = strip_indent(line, 3)?;
    let marker = *rest.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let len = rest.bytes().take_while(|byte| *byte == marker).count();
    if len < 3 {
        return None;
    }
    if marker == b'`' && rest[len..].contains('`') {
        return None;
    }
    Some(Fence { marker, len })
}

fn fence_closes(line: &str, fence: &Fence) -> bool {
    let Some(rest) = strip_indent(line, 3) else {
        return false;
    };
    let len = rest
        .bytes()
        .take_while(|byte| *byte == fence.marker)
        .count();
    len >= fence.len && rest[len..].trim().is_empty()
}

fn split_blocks(text: &str, base: usize) -> Vec<Range<usize>> {
    let mut blocks = Vec::new();
    let mut start: Option<usize> = None;
    let mut fence: Option<Fence> = None;
    let mut blank_before = false;
    let mut offset = 0;

    for line in text.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();
        let content = line.trim_end_matches(['\n', '\r']);

        if let Some(open) = &fence {
            if fence_closes(content, open) {
                fence = None;
            }
            continue;
        }
        if content.trim().is_empty() {
            blank_before = start.is_some();
            continue;
        }
        let unindented = !content.starts_with([' ', '\t']);
        match start {
            Some(block_start) if blank_before && unindented => {
                blocks.push(base + block_start..base + line_start);
                start = Some(line_start);
            },
            Some(_) => {},
            None => start = Some(line_start),
        }
        blank_before = false;
        fence = fence_open(content);
    }
    if let Some(block_start) = start {
        blocks.push(base + block_start..base + text.len());
    }
    blocks
}

fn has_reference_definitions(text: &str) -> bool {
    text.lines().any(|line| {
        let Some(rest) = strip_indent(line, 3) else {
            return false;
        };
        let Some(label) = rest.strip_prefix('[') else {
            return false;
        };
        label
            .find("]:")
            .is_some_and(|end| end > 0 && !label[..end].contains(']'))
    })
}
