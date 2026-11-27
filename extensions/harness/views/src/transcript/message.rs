//! One row of the transcript list: a user prompt with its attachments, or an
//! assistant reply as its visible parts in order, the admitted extensions'
//! marks under it, and the per-message actions shown on hover or keyboard
//! selection.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use gpui_kit::{
    AnyElement, Context, FollowMode, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    list, px,
};
use harness_store::{AssistantMessage, Message, MessageErrorKind, MessageWithParts, PartBody};
use harness_ui::IconName;
use harness_ui::components::{Button, EmptyState, Spinner};
use harness_ui::theme::{radius, size, space, text as type_scale, tokens};

use crate::strings::transcript as text;
use crate::views::transcript::{TranscriptView, rows};

const CONTENT_WIDTH: f32 = 880.0;

pub(super) struct RowFlags {
    pub(super) group: SharedString,
    pub(super) selected: bool,
    pub(super) busy: bool,
}

impl TranscriptView {
    pub(super) fn render_body(&self, cx: &Context<Self>) -> AnyElement {
        let loaded = self.session.read(cx).model().loaded;
        if !loaded {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(Spinner::new())
                .into_any_element();
        }
        if self.rows.is_empty() {
            return div()
                .flex_1()
                .child(
                    EmptyState::new(text::EMPTY_TITLE)
                        .icon(IconName::Sparkle)
                        .description(text::EMPTY_BODY),
                )
                .into_any_element();
        }
        let items = list(
            self.list.clone(),
            cx.processor(|this, index: usize, window, cx| this.render_row(index, window, cx)),
        )
        .size_full();
        let mut area = div().relative().flex_1().min_h_0().child(items);
        if !self.following {
            area = area.child(
                div().absolute().bottom(space::LG).right(space::XL).child(
                    Button::new("transcript-jump-latest")
                        .label(text::JUMP_TO_LATEST)
                        .icon(IconName::ChevronDown)
                        .small()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.list.set_follow_mode(FollowMode::Tail);
                            this.following = true;
                            cx.notify();
                        })),
                ),
            );
        }
        area.into_any_element()
    }

    fn render_row(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(key) = self.rows.get(index).cloned() else {
            return div().into_any_element();
        };
        let found = {
            let model = self.session.read(cx).model();
            let messages = model.visible_messages();
            let live = model.is_busy() && key.index + 1 == messages.len();
            messages
                .get(key.index)
                .cloned()
                .map(|entry| (entry, live, model.is_busy()))
        };
        let Some((entry, live, busy)) = found else {
            return div().into_any_element();
        };
        let content = match &entry.message {
            Message::User(user) if user.system => Self::render_compaction_note(&entry, cx),
            Message::User(_) => Self::render_user(&entry, cx),
            Message::Assistant(reply) => self.render_assistant(&entry, reply, live, cx),
        };
        let adornments = self.render_adornments(&entry, window, cx);
        let palette = *tokens(cx);
        let selected = self.selected == Some(index);
        let group = SharedString::from(format!("transcript-row-{index}"));
        let flags = RowFlags {
            group: group.clone(),
            selected,
            busy,
        };
        let actions = Self::render_actions(index, &entry, flags, cx);
        let row = div()
            .id(("transcript-row", index))
            .group(group)
            .relative()
            .w_full()
            .px(space::XL)
            .py(space::SM)
            .border_l_2()
            .border_color(palette.surface.app.with_alpha(0.0))
            .on_click(cx.listener(move |this, _, window, cx| this.select(index, window, cx)))
            .child(
                div()
                    .w_full()
                    .max_w(px(CONTENT_WIDTH))
                    .mx_auto()
                    .child(content)
                    .children(adornments),
            )
            .child(actions);
        let row = if selected {
            row.border_color(palette.border.focus)
                .bg(palette.surface.panel)
        } else {
            row
        };
        row.into_any_element()
    }

    fn render_user(entry: &MessageWithParts, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let chips: Vec<AnyElement> = entry
            .parts
            .iter()
            .filter_map(|part| match &part.body {
                PartBody::File { name, url, .. } => Some(file_chip(name.as_deref(), url, cx)),
                _ => None,
            })
            .collect();
        let body = rows::plain_text(entry);
        div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .w_full()
            .px(space::LG)
            .py(space::MD)
            .rounded(radius::LG)
            .border_1()
            .border_color(palette.border.subtle)
            .bg(palette.surface.raised)
            .child(
                div()
                    .text_size(type_scale::CAPTION)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text.muted)
                    .child(text::YOU),
            )
            .children((!body.is_empty()).then(|| {
                div()
                    .text_size(type_scale::BODY)
                    .text_color(palette.text.primary)
                    .child(SharedString::from(body))
            }))
            .children((!chips.is_empty()).then(|| {
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(space::XS)
                    .children(chips)
            }))
            .into_any_element()
    }

    fn render_assistant(
        &mut self,
        entry: &MessageWithParts,
        reply: &AssistantMessage,
        live: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        if reply.system {
            return self.render_summary(entry, cx);
        }
        let streaming = live
            .then(|| {
                entry.parts.iter().rev().find(|part| {
                    matches!(
                        &part.body,
                        PartBody::Text {
                            synthetic: false,
                            ..
                        } | PartBody::Reasoning { .. }
                    )
                })
            })
            .flatten()
            .map(|part| part.id.clone());
        let parts: Vec<AnyElement> = entry
            .parts
            .iter()
            .filter_map(|part| self.render_part(part, streaming.as_ref(), cx))
            .collect();
        let palette = *tokens(cx);
        let mut column = div().flex().flex_col().gap(space::SM).w_full();
        let empty = parts.is_empty();
        column = column.children(parts);
        if live && empty {
            column = column.child(working_line(cx));
        }
        if let Some(error) = &reply.error {
            let (color, message) = if error.kind == MessageErrorKind::Aborted {
                (palette.text.muted, text::STOPPED.to_owned())
            } else {
                (palette.danger.fg, error.message.clone())
            };
            column = column.child(
                div()
                    .text_size(type_scale::SMALL)
                    .text_color(color)
                    .child(SharedString::from(message)),
            );
        }
        column.into_any_element()
    }
}

pub(super) fn working_line(cx: &Context<TranscriptView>) -> AnyElement {
    let palette = *tokens(cx);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::SM)
        .text_size(type_scale::SMALL)
        .text_color(palette.text.muted)
        .child(Spinner::new().size(size::ICON_SM))
        .child(text::WORKING)
        .into_any_element()
}

pub(super) fn file_chip(name: Option<&str>, url: &str, cx: &Context<TranscriptView>) -> AnyElement {
    let palette = *tokens(cx);
    let label = match name {
        Some(name) => name.to_owned(),
        None if url.starts_with("data:") => text::ATTACHMENT.to_owned(),
        None => url
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(text::ATTACHMENT)
            .to_owned(),
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(space::XS)
        .h(px(20.))
        .px(space::SM)
        .rounded(radius::SM)
        .border_1()
        .border_color(palette.border.subtle)
        .bg(palette.surface.sunken)
        .text_size(type_scale::CAPTION)
        .text_color(palette.text.secondary)
        .child(harness_ui::Icon::new(IconName::File).size(px(12.)))
        .child(
            div()
                .max_w(px(240.))
                .truncate()
                .child(SharedString::from(label)),
        )
        .into_any_element()
}
