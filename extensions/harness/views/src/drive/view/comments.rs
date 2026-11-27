//! The Comments tab: the composer (a new comment pinned to a paragraph, or a
//! reply to a thread) and the threads, open ones first, each with Reply and
//! Resolve.

use crate::drive::items::CommentThread;
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, div,
};
use systemprompt_harness_shared::components::{
    Badge, Button, Card, IconButton, Select, SelectOption, Spinner,
};
use systemprompt_harness_shared::theme::{space, text as type_scale, tokens};
use systemprompt_harness_shared::{IconName, Tone};
use systemprompt_models::documents::client::DriveComment;

use super::preview_parts::muted;
use super::{DriveView, OpenDocument};
use crate::drive::model::DriveLoad;
use crate::drive::strings as text;

impl DriveView {
    pub(super) fn render_comments(&self, open: &OpenDocument, cx: &Context<Self>) -> AnyElement {
        let threads: &Vec<CommentThread> = match &open.comments {
            DriveLoad::Ready(threads) => threads,
            DriveLoad::Loading | DriveLoad::Idle => return Spinner::new().into_any_element(),
            DriveLoad::Failed(failure) => return muted(text::failure(*failure), cx),
        };
        let mut ordered: Vec<(usize, &CommentThread)> = threads.iter().enumerate().collect();
        ordered.sort_by_key(|(_, thread)| thread.is_resolved());
        let mut column = div()
            .flex()
            .flex_col()
            .gap(space::MD)
            .child(self.composer(open, cx));
        if threads.is_empty() {
            column = column.child(muted(text::NO_COMMENTS, cx));
        }
        column
            .children(
                ordered
                    .into_iter()
                    .map(|(index, thread)| self.thread(index, thread, cx)),
            )
            .into_any_element()
    }

    fn composer(&self, open: &OpenDocument, cx: &Context<Self>) -> AnyElement {
        let palette = tokens(cx);
        let target = match &self.reply_to {
            Some(_) => div()
                .flex()
                .flex_row()
                .items_center()
                .gap(space::SM)
                .text_size(type_scale::SMALL)
                .text_color(palette.text.secondary)
                .child(text::REPLY)
                .child(
                    IconButton::new("drive-cancel-reply", IconName::X, text::CANCEL_REPLY)
                        .small()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.reply_to(None, window, cx)),
                        ),
                )
                .into_any_element(),
            None => self.anchor_select(open, cx),
        };
        div()
            .flex()
            .flex_col()
            .gap(space::SM)
            .child(target)
            .child(self.comment_field.clone())
            .child(
                div().flex().flex_row().justify_end().child(
                    Button::new("drive-send-comment")
                        .label(if self.reply_to.is_some() {
                            text::REPLY
                        } else {
                            text::COMMENT
                        })
                        .primary()
                        .small()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.submit_comment(window, cx)),
                        ),
                ),
            )
            .into_any_element()
    }

    fn anchor_select(&self, open: &OpenDocument, cx: &Context<Self>) -> AnyElement {
        let palette = tokens(cx);
        let anchors = open
            .preview
            .ready()
            .map(|preview| preview.anchors.clone())
            .unwrap_or_default();
        if anchors.is_empty() {
            return muted(text::COMMENT_NEEDS_ANCHOR, cx);
        }
        let options = anchors
            .into_iter()
            .map(|anchor| SelectOption::new(anchor.node, anchor.excerpt));
        let select = Select::new("drive-comment-anchor", options)
            .selected(self.anchor.clone().map(SharedString::from))
            .small()
            .on_select(cx.listener(|this, node: &SharedString, _, cx| {
                this.set_anchor(node.to_string(), cx);
            }));
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .text_size(type_scale::SMALL)
            .text_color(palette.text.secondary)
            .child(text::COMMENT_ON)
            .child(div().flex_1().min_w_0().child(select))
            .into_any_element()
    }

    fn thread(&self, index: usize, thread: &CommentThread, cx: &Context<Self>) -> AnyElement {
        let resolved = thread.is_resolved();
        let mut actions = div().flex().flex_row().gap(space::XS);
        if resolved {
            actions = actions.child(Badge::new(text::RESOLVED).tone(Tone::Success));
        } else {
            let reply_to = thread.root.id.clone();
            let resolve = thread.root.id.clone();
            actions = actions
                .child(
                    Button::new(("drive-thread-reply", index))
                        .label(text::REPLY)
                        .ghost()
                        .small()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.reply_to(Some(reply_to.clone()), window, cx);
                        })),
                )
                .child(
                    Button::new(("drive-thread-resolve", index))
                        .label(text::RESOLVE)
                        .icon(IconName::Check)
                        .ghost()
                        .small()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.resolve_comment(resolve.clone(), cx);
                        })),
                );
        }
        let selected = self.reply_to.as_ref() == Some(&thread.root.id);
        let mut card = Card::new()
            .padding(space::MD)
            .child(comment(&thread.root, cx));
        for reply in &thread.replies {
            card = card.child(div().pl(space::XL).child(comment(reply, cx)));
        }
        let card = card.child(actions);
        let frame = div()
            .rounded(systemprompt_harness_shared::theme::radius::XL)
            .border_1();
        let border = if selected {
            tokens(cx).border.focus
        } else {
            tokens(cx).surface.app.with_alpha(0.0)
        };
        frame.border_color(border).child(card).into_any_element()
    }
}

fn comment(comment: &DriveComment, cx: &Context<DriveView>) -> AnyElement {
    let palette = tokens(cx);
    let quote = comment
        .anchor
        .as_ref()
        .and_then(|anchor| anchor.find.clone());
    div()
        .flex()
        .flex_col()
        .gap(space::XXS)
        .children(quote.map(|quote| {
            div()
                .text_size(type_scale::SMALL)
                .text_color(palette.text.muted)
                .truncate()
                .child(SharedString::from(quote))
        }))
        .child(
            div()
                .text_size(type_scale::BODY)
                .child(SharedString::from(comment.body.clone())),
        )
        .child(
            div()
                .text_size(type_scale::CAPTION)
                .text_color(palette.text.muted)
                .child(text::when(&comment.created_at, chrono::Local::now())),
        )
        .into_any_element()
}
