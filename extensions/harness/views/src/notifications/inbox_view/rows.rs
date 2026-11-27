//! The inbox body.
//!
//! The explicit withheld, unserved, connecting, unavailable and empty states,
//! or the notifications newest first with each row's unread mark, attention,
//! age and Mark read, then Load older.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use chrono::{DateTime, Utc};
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement as _, Styled as _, div};
use harness_ui::components::{Badge, Button, EmptyState, IconButton, ListRow, Spinner, StatusDot};
use harness_ui::theme::{size, space, text as type_scale, tokens};
use harness_ui::{IconName, Tone};
use sp_notifications::{Inbox, Notification};

use super::InboxView;
use crate::model::{Action, InboxBody, accepts_actions, inbox_body, needs_reader};
use crate::strings as text;

#[derive(Clone, Copy)]
struct RowFrame {
    actionable: bool,
    now: DateTime<Utc>,
}

impl InboxView {
    pub(super) fn render_body(&self, inbox: &Inbox, cx: &Context<Self>) -> AnyElement {
        let status = inbox.status();
        match inbox_body(status, !inbox.items().is_empty()) {
            InboxBody::Withheld => EmptyState::new(text::WITHHELD_TITLE)
                .icon(IconName::Shield)
                .description(text::WITHHELD_BODY)
                .into_any_element(),
            InboxBody::NotServed => EmptyState::new(text::NOT_SERVED_TITLE)
                .icon(IconName::CircleX)
                .description(text::NOT_SERVED_BODY)
                .into_any_element(),
            InboxBody::Connecting => div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(space::SM)
                .p(space::XL)
                .text_size(type_scale::SMALL)
                .text_color(tokens(cx).text.muted)
                .child(Spinner::new())
                .child(text::CONNECTING_TITLE)
                .into_any_element(),
            InboxBody::Unavailable => EmptyState::new(text::UNAVAILABLE_TITLE)
                .icon(IconName::Alert)
                .description(text::feed_line(status, Utc::now()))
                .into_any_element(),
            InboxBody::Empty => EmptyState::new(text::EMPTY_TITLE)
                .icon(IconName::CircleCheck)
                .description(text::EMPTY_BODY)
                .into_any_element(),
            InboxBody::List => self.render_list(inbox, cx),
        }
    }

    fn render_list(&self, inbox: &Inbox, cx: &Context<Self>) -> AnyElement {
        let frame = RowFrame {
            actionable: accepts_actions(inbox.status()),
            now: Utc::now(),
        };
        let actionable = frame.actionable;
        let rows = inbox
            .items()
            .iter()
            .enumerate()
            .map(|(index, item)| self.render_row(index, item, frame, cx));
        let footer = if inbox.older_cursor().is_some() {
            Button::new("notifications-load-older")
                .label(text::LOAD_OLDER)
                .ghost()
                .small()
                .loading(self.pending.is_running(&Action::LoadOlder))
                .disabled(!actionable)
                .on_click(cx.listener(|this, _, _, cx| this.act(Action::LoadOlder, cx)))
                .into_any_element()
        } else {
            div()
                .text_size(type_scale::SMALL)
                .text_color(tokens(cx).text.muted)
                .child(text::NO_OLDER)
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .gap(space::XXS)
            .children(rows)
            .child(div().flex().justify_center().pt(space::MD).child(footer))
            .into_any_element()
    }

    fn render_row(
        &self,
        index: usize,
        item: &Notification,
        frame: RowFrame,
        cx: &Context<Self>,
    ) -> AnyElement {
        let unread = item.read_at.is_none();
        let leading = if unread {
            StatusDot::new(("notification-unread", index), Tone::Accent).into_any_element()
        } else {
            div().size(size::ICON_SM).into_any_element()
        };
        let mark_read = unread.then(|| {
            let id = item.id.clone();
            IconButton::new(
                ("notification-mark-read", index),
                IconName::Check,
                text::MARK_READ,
            )
            .small()
            .loading(self.pending.is_running(&Action::MarkRead(id.clone())))
            .disabled(!frame.actionable)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.act(Action::MarkRead(id.clone()), cx);
            }))
        });
        let trailing = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .text_size(type_scale::CAPTION)
            .text_color(tokens(cx).text.muted)
            .children(
                needs_reader(item).then(|| Badge::new(text::NEEDS_YOU).tone(Tone::Warning).pill()),
            )
            .children(text::coalesced(item.coalesce_count))
            .child(text::relative(item.created_at, frame.now))
            .children(mark_read);
        let mut row = ListRow::new(("notification", index), item.title.clone())
            .tall()
            .leading(leading)
            .trailing(trailing);
        if !item.body.is_empty() {
            row = row.subtitle(item.body.clone());
        }
        row.into_any_element()
    }
}
