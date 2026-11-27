//! The action bar shown on a hovered or selected message: copy, revert and
//! fork.

use super::message::RowFlags;
use crate::strings::transcript as text;
use crate::views::transcript::TranscriptView;
use gpui_kit::{
    AnyElement, Context, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div,
};
use harness_store::{Message, MessageWithParts};
use harness_ui::IconName;
use harness_ui::components::IconButton;
use harness_ui::theme::{radius, space, tokens};

impl TranscriptView {
    pub(super) fn render_actions(
        index: usize,
        entry: &MessageWithParts,
        flags: RowFlags,
        cx: &Context<Self>,
    ) -> AnyElement {
        let RowFlags {
            group,
            selected,
            busy,
        } = flags;
        let palette = *tokens(cx);
        let is_user = matches!(entry.message, Message::User(_));
        let mut bar = div()
            .absolute()
            .top(space::XS)
            .right(space::XL)
            .flex()
            .flex_row()
            .gap(space::XXS)
            .p(space::XXS)
            .rounded(radius::MD)
            .border_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.raised)
            .child(
                IconButton::new(("transcript-copy", index), IconName::Copy, text::COPY)
                    .small()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.copy_message(index, window, cx);
                    })),
            );
        if is_user {
            bar = bar.child(
                IconButton::new(("transcript-revert", index), IconName::Undo, text::REVERT)
                    .small()
                    .disabled(busy)
                    .on_click(cx.listener(move |this, _, _, cx| this.revert_message(index, cx))),
            );
        }
        bar = bar.child(
            IconButton::new(("transcript-fork", index), IconName::Branch, text::FORK)
                .small()
                .on_click(cx.listener(move |this, _, _, cx| this.fork_message(index, cx))),
        );
        if !selected {
            bar = bar
                .opacity(0.)
                .group_hover(group, |style| style.opacity(1.));
        }
        bar.into_any_element()
    }
}
