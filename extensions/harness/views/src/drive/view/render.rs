//! The Drive screen drawn: folders, the document list, the preview and the sync
//! panel, or the withheld state. Notices the screen raised show as toasts.

use super::{DriveView, FOLDER_PANE, LIST_PANE};
use crate::drive::strings as text;
use gpui_kit::{Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div, px};
use systemprompt_harness_shared::IconName;
use systemprompt_harness_shared::components::{EmptyState, Toast};
use systemprompt_harness_shared::theme::tokens;

impl Render for DriveView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        for (tone, message) in self.notices.drain(..) {
            window.defer(cx, move |window, cx| {
                Toast::new(tone, message).show(window, cx);
            });
        }
        if std::mem::take(&mut self.clear_comment) {
            self.comment_field
                .update(cx, |field, cx| field.set_text("", window, cx));
        }
        let palette = *tokens(cx);
        if self.target.is_none() {
            return div().size_full().child(
                EmptyState::new(text::WITHHELD_TITLE)
                    .icon(IconName::Cloud)
                    .description(text::WITHHELD_BODY),
            );
        }
        let folders = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(px(FOLDER_PANE))
            .h_full()
            .border_r_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.panel)
            .child(self.render_folders(cx))
            .child(self.render_sync(cx));
        let list = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(px(LIST_PANE))
            .h_full()
            .border_r_1()
            .border_color(palette.border.subtle)
            .child(self.render_list(cx));
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(folders)
            .child(list)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.render_preview(cx)),
            )
    }
}
