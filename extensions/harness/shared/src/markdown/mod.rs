//! Markdown rendering for transcripts and documents.
//!
//! Parsing, layout, tables, task lists and fenced-code highlighting come from
//! the toolkit's text view; this module adds what a streaming transcript needs
//! on top: the text is held as top-level blocks ([`MarkdownDocument`]), each
//! block is rendered by its own text view keyed by content hash, so a growing
//! reply re-parses only its last block.

mod blocks;

use std::sync::Arc;

use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::text::TextView;
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, Styled as _, Window, div,
};

pub use blocks::{MarkdownBlock, MarkdownDocument};

use crate::theme::{space, text, tokens};

type LinkHandler = Arc<dyn Fn(&SharedString, &mut Window, &mut App) + Send + Sync>;

/// Rendered markdown: one text view per block of a [`MarkdownDocument`].
///
/// `streaming(true)` fades in words appended to the last block. `on_link_click`
/// replaces the toolkit opening the URL. `copy_button(tooltip)` adds a copy
/// button to fenced code blocks. `muted(true)` uses the secondary text colour.
/// `from_text` splits on every render; keep a `MarkdownDocument` for text that
/// changes.
#[derive(IntoElement)]
pub struct Markdown {
    id: ElementId,
    blocks: Vec<MarkdownBlock>,
    streaming: bool,
    selectable: bool,
    muted: bool,
    on_link: Option<LinkHandler>,
    copy_tooltip: Option<SharedString>,
}

impl std::fmt::Debug for Markdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Markdown")
            .field("id", &self.id)
            .field("blocks", &self.blocks.len())
            .field("streaming", &self.streaming)
            .finish_non_exhaustive()
    }
}

impl Markdown {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, document: &MarkdownDocument) -> Self {
        Self {
            id: id.into(),
            blocks: document.blocks().to_vec(),
            streaming: false,
            selectable: true,
            muted: false,
            on_link: None,
            copy_tooltip: None,
        }
    }

    #[must_use]
    pub fn from_text(id: impl Into<ElementId>, text: &str) -> Self {
        Self::new(id, &MarkdownDocument::from_text(text))
    }

    #[must_use]
    pub const fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    #[must_use]
    pub const fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    #[must_use]
    pub const fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    #[must_use]
    pub fn on_link_click(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_link = Some(Arc::new(handler));
        self
    }

    #[must_use]
    pub fn copy_button(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.copy_tooltip = Some(tooltip.into());
        self
    }
}

fn block_view(
    block: &MarkdownBlock,
    streaming: bool,
    selectable: bool,
    on_link: Option<LinkHandler>,
    copy_tooltip: Option<SharedString>,
) -> TextView {
    let mut view = TextView::markdown(ElementId::Integer(block.key()), block.text().clone())
        .selectable(selectable)
        .stream_fade(streaming && block.is_tail());
    if let Some(handler) = on_link {
        view = view.on_link_click(move |url, _event, window, cx| handler(url, window, cx));
    }
    if let Some(tooltip) = copy_tooltip {
        view = view.code_block_actions(move |code_block, _window, _cx| {
            let code = code_block.code();
            Clipboard::new(ElementId::Integer(fingerprint(&code)))
                .tooltip(tooltip.clone())
                .value(code)
        });
    }
    view
}

fn fingerprint(text: &str) -> u64 {
    use std::hash::{Hash as _, Hasher as _};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

impl RenderOnce for Markdown {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = tokens(cx);
        let color = if self.muted {
            palette.text.secondary
        } else {
            palette.text.primary
        };
        let streaming = self.streaming;
        let selectable = self.selectable;
        let on_link = self.on_link;
        let copy_tooltip = self.copy_tooltip;
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .gap(space::MD)
            .text_size(text::BODY)
            .text_color(color)
            .children(self.blocks.into_iter().map(|block| {
                block_view(
                    &block,
                    streaming,
                    selectable,
                    on_link.clone(),
                    copy_tooltip.clone(),
                )
            }))
    }
}
