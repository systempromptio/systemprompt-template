//! Unified diff rendering: a file-by-file view with numbered, tinted lines and
//! folded unchanged runs, and an inline `+n −m` stat.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod model;
mod parse;
mod rows;
mod stat;

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::{
    AnyElement, App, ElementId, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
};

pub use model::{
    DiffLine, FilePatch, FileStatus, Hunk, LineKind, Patch, Segment, Stat, fold_unchanged,
};
pub use stat::DiffStat;

use crate::icons::{Icon, IconName};
use crate::theme::{self, radius, space, text};
use rows::{RowStyle, file_header, hunk_header, line_row};

const DEFAULT_CONTEXT: usize = 3;
const DEFAULT_MIN_FOLD: usize = 4;

type FoldLabel = Rc<dyn Fn(usize) -> SharedString>;

#[derive(Default)]
struct ExpandedFolds(HashSet<u64>);

const fn fold_key(file: usize, hunk: usize, start: usize) -> u64 {
    ((file as u64) << 42) | ((hunk as u64) << 21) | (start as u64 & 0x1F_FFFF)
}

struct HunkRender<'a> {
    file: usize,
    hunk: usize,
    style: RowStyle<'a>,
    expanded: &'a HashSet<u64>,
    folds: &'a Entity<ExpandedFolds>,
}

/// A read-only unified diff, file by file.
///
/// Keep the parsed `Arc<Patch>` between frames. Unchanged runs fold to
/// `context_lines` of context and expand on click; `fold_label` supplies the
/// folded row's text from the hidden count. Long lines scroll horizontally per
/// hunk; `max_height` makes the whole view scroll vertically.
#[derive(IntoElement)]
pub struct DiffView {
    id: ElementId,
    patch: Arc<Patch>,
    max_height: Option<Pixels>,
    file_headers: bool,
    context: usize,
    min_fold: usize,
    fold_label: Option<FoldLabel>,
}

impl std::fmt::Debug for DiffView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiffView")
            .field("id", &self.id)
            .field("files", &self.patch.files.len())
            .field("max_height", &self.max_height)
            .finish_non_exhaustive()
    }
}

impl DiffView {
    #[must_use]
    pub fn new(id: impl Into<ElementId>, patch: Arc<Patch>) -> Self {
        Self {
            id: id.into(),
            patch,
            max_height: None,
            file_headers: true,
            context: DEFAULT_CONTEXT,
            min_fold: DEFAULT_MIN_FOLD,
            fold_label: None,
        }
    }

    #[must_use]
    pub fn from_text(id: impl Into<ElementId>, patch: &str) -> Self {
        Self::new(id, Arc::new(Patch::parse(patch)))
    }

    #[must_use]
    pub const fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = Some(height);
        self
    }

    #[must_use]
    pub const fn file_headers(mut self, show: bool) -> Self {
        self.file_headers = show;
        self
    }

    #[must_use]
    pub const fn context_lines(mut self, lines: usize) -> Self {
        self.context = lines;
        self
    }

    #[must_use]
    pub fn fold_label(mut self, label: impl Fn(usize) -> SharedString + 'static) -> Self {
        self.fold_label = Some(Rc::new(label));
        self
    }
}

impl DiffView {
    fn hunk_body(&self, hunk: &Hunk, render: &HunkRender<'_>) -> AnyElement {
        let mut body = div().flex().flex_col().min_w_full();
        for segment in fold_unchanged(&hunk.lines, self.context, self.min_fold) {
            match segment {
                Segment::Lines(range) => {
                    for line in &hunk.lines[range] {
                        body = body.child(line_row(line, &render.style));
                    }
                },
                Segment::Folded(range) => {
                    let key = fold_key(render.file, render.hunk, range.start);
                    if render.expanded.contains(&key) {
                        for line in &hunk.lines[range] {
                            body = body.child(line_row(line, &render.style));
                        }
                    } else {
                        body = body.child(self.fold_row(range.len(), key, render));
                    }
                },
            }
        }
        div()
            .id(ElementId::NamedInteger(
                SharedString::new_static("hunk"),
                fold_key(render.file, render.hunk, 0),
            ))
            .w_full()
            .overflow_x_scroll()
            .child(body)
            .into_any_element()
    }

    fn fold_row(&self, hidden: usize, key: u64, render: &HunkRender<'_>) -> AnyElement {
        let palette = render.style.palette;
        let label = self.fold_label.as_ref().map_or_else(
            || SharedString::from(hidden.to_string()),
            |label| label(hidden),
        );
        let folds = render.folds.clone();
        div()
            .id(ElementId::NamedInteger(
                SharedString::new_static("fold"),
                key,
            ))
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .w_full()
            .pl(render.style.gutter)
            .py(space::XXS)
            .bg(palette.diff.hunk_bg)
            .text_size(text::SMALL)
            .text_color(palette.text.muted)
            .cursor_pointer()
            .hover(|this| this.text_color(palette.text.primary))
            .tab_index(0)
            .focus_visible(|this| this.bg(palette.surface.hover))
            .on_click(move |_, _, cx| {
                folds.update(cx, |state, cx| {
                    state.0.insert(key);
                    cx.notify();
                });
            })
            .child(Icon::new(IconName::Ellipsis).small())
            .child(label)
            .into_any_element()
    }
}

impl RenderOnce for DiffView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let folds = window.use_keyed_state(
            ElementId::NamedChild(Arc::new(self.id.clone()), SharedString::new_static("folds")),
            cx,
            |_, _| ExpandedFolds::default(),
        );
        let expanded = folds.read(cx).0.clone();
        let palette = *theme::tokens(cx);
        let mono = theme::mono_font(cx);

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .rounded(radius::LG)
            .border_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.sunken)
            .overflow_hidden();
        if let Some(height) = self.max_height {
            root = root.max_h(height).overflow_y_scroll();
        }

        for (file_index, file) in self.patch.files.iter().enumerate() {
            if self.file_headers {
                root = root.child(file_header(file, &palette));
            }
            for (hunk_index, hunk) in file.hunks.iter().enumerate() {
                let render = HunkRender {
                    file: file_index,
                    hunk: hunk_index,
                    style: RowStyle::for_hunk(hunk, &palette, mono.clone()),
                    expanded: &expanded,
                    folds: &folds,
                };
                root = root.child(hunk_header(hunk, &render.style));
                root = root.child(self.hunk_body(hunk, &render));
            }
        }
        root
    }
}
