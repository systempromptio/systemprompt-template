//! The session header (title, todo progress and the changed-lines summary) and
//! the changes panel it opens, one diff per changed file.

use gpui_kit::{
    AnyElement, Context, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, div, px,
};
use systemprompt_engine::store::TodoStatus;
use systemprompt_engine::workspace::FileDiff;
use systemprompt_harness_shared::components::{Badge, EmptyState, IconButton, SectionHeader};
use systemprompt_harness_shared::theme::{radius, size, space, text as type_scale, tokens};
use systemprompt_harness_shared::{DiffStat, DiffView, Icon, IconName};

use crate::strings::transcript as text;
use crate::transcript::TranscriptView;

const PANEL_WIDTH: f32 = 480.0;
const HEADER_HEIGHT: f32 = 36.0;

fn count(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

impl TranscriptView {
    pub(super) fn render_header(&self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let (title, diff, done, total) = {
            let model = self.session.read(cx).model();
            let done = model
                .todos
                .iter()
                .filter(|todo| todo.status == TodoStatus::Completed)
                .count();
            (
                model.title().unwrap_or(text::UNTITLED).to_owned(),
                model.diff(),
                done,
                model.todos.len(),
            )
        };
        let mut row = div()
            .flex()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .gap(space::MD)
            .h(px(HEADER_HEIGHT))
            .px(space::XL)
            .border_b_1()
            .border_color(palette.border.subtle)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(type_scale::BODY)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text.primary)
                    .child(SharedString::from(title)),
            );
        if total > 0 {
            row = row.child(Badge::new(text::todo_progress(done, total)).icon(IconName::ListTodo));
        }
        if diff.files > 0 || self.changes_open {
            let toggle = div()
                .id("transcript-changes-toggle")
                .flex()
                .flex_row()
                .items_center()
                .gap(space::SM)
                .px(space::SM)
                .py(space::XXS)
                .rounded(radius::MD)
                .cursor_pointer()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.secondary)
                .hover(|style| style.bg(palette.surface.hover))
                .on_click(cx.listener(|this, _, _, cx| this.toggle_changes(cx)))
                .child(Icon::new(IconName::File).size(size::ICON_SM))
                .child(SharedString::from(text::files_changed(diff.files)))
                .child(DiffStat::new(count(diff.additions), count(diff.deletions)));
            let toggle = if self.changes_open {
                toggle.bg(palette.surface.selected)
            } else {
                toggle
            };
            row = row.child(toggle);
        }
        row.into_any_element()
    }

    pub(super) fn toggle_changes(&mut self, cx: &mut Context<Self>) {
        self.changes_open = !self.changes_open;
        if self.changes_open {
            self.session
                .update(cx, crate::state::session_state::SessionState::load_diffs);
        }
        cx.notify();
    }

    pub(super) fn render_changes(&mut self, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let diffs: Vec<FileDiff> = self.session.read(cx).diffs().to_vec();
        self.file_patches
            .retain(|path| diffs.iter().any(|diff| &diff.path == path));
        let session = self.session.clone();
        let actions = div()
            .flex()
            .flex_row()
            .gap(space::XXS)
            .child(
                IconButton::new("changes-refresh", IconName::Sync, text::CHANGES_REFRESH)
                    .small()
                    .on_click(move |_, _, cx| {
                        session.update(cx, crate::state::session_state::SessionState::load_diffs);
                    }),
            )
            .child(
                IconButton::new("changes-close", IconName::X, text::CHANGES_CLOSE)
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_changes(cx))),
            );
        let body = if diffs.is_empty() {
            div()
                .flex_1()
                .child(EmptyState::new(text::CHANGES_EMPTY).icon(IconName::File))
                .into_any_element()
        } else {
            let files: Vec<AnyElement> = diffs
                .iter()
                .map(|diff| self.render_file(diff, cx))
                .collect();
            div()
                .id("changes-scroll")
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .gap(space::LG)
                .p(space::MD)
                .overflow_y_scroll()
                .children(files)
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(px(PANEL_WIDTH))
            .h_full()
            .border_l_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.panel)
            .child(
                SectionHeader::new(text::CHANGES_TITLE)
                    .count(diffs.len())
                    .trailing(actions),
            )
            .child(body)
            .into_any_element()
    }

    fn render_file(&mut self, diff: &FileDiff, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let shown_path = diff.path.to_string_lossy().replace('\\', "/");
        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .text_size(type_scale::SMALL)
            .child(
                Icon::new(IconName::File)
                    .size(size::ICON_SM)
                    .color(palette.text.muted),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(palette.text.primary)
                    .child(SharedString::from(shown_path.clone())),
            )
            .child(DiffStat::new(diff.additions, diff.deletions));
        let content = if diff.binary || diff.patch.trim().is_empty() {
            div()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.muted)
                .child(text::BINARY_FILE)
                .into_any_element()
        } else {
            let patch = self.file_patches.patch(&diff.path, &diff.patch);
            DiffView::new(
                ElementId::Name(SharedString::from(format!("changes-{shown_path}"))),
                patch,
            )
            .file_headers(false)
            .fold_label(|hidden| SharedString::from(text::folded(hidden)))
            .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .child(header)
            .child(content)
            .into_any_element()
    }
}
