//! The inbox screen.
//!
//! The reader's notifications newest first, read straight from the handle's
//! feed, with mark read, mark all read, load older and a check for new ones.
//! The view keeps only what is its own: the actions in flight, the last
//! action's failure and which items it already reported seen.

mod rows;

use std::collections::HashSet;

use chrono::Utc;
use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, StatefulInteractiveElement as _, Styled as _, Task, Window, div, px,
};
use harness_ui::IconName;
use harness_ui::components::{Banner, Button, IconButton, StatusDot};
use harness_ui::theme::{space, text as type_scale, tokens};
use sp_notifications::{FeedStatus, Inbox, NotificationId, NotificationsHandle};
use tokio::runtime::Handle;
use tokio::sync::watch;

use crate::model::{Action, Pending, accepts_actions, feed_tone, unseen};
use crate::{bridge, strings as text};

const PAGE_WIDTH: f32 = 760.0;

pub(crate) struct InboxView {
    handle: NotificationsHandle,
    feed: watch::Receiver<Inbox>,
    runtime: Handle,
    pending: Pending,
    failure: Option<&'static str>,
    reported_seen: HashSet<NotificationId>,
    tasks: Vec<Task<()>>,
    _follow: Task<()>,
}

impl InboxView {
    pub(crate) fn new(handle: NotificationsHandle, runtime: Handle, cx: &Context<Self>) -> Self {
        let follow = bridge::follow(cx, handle.subscribe(), |this: &mut Self, cx| {
            this.report_seen(cx);
            cx.notify();
        });
        let mut view = Self {
            feed: handle.subscribe(),
            handle,
            runtime,
            pending: Pending::default(),
            failure: None,
            reported_seen: HashSet::new(),
            tasks: Vec::new(),
            _follow: follow,
        };
        view.report_seen(cx);
        view
    }

    fn keep(&mut self, task: Task<()>) {
        self.tasks.retain(|task| !task.is_ready());
        self.tasks.push(task);
    }

    fn report_seen(&mut self, cx: &Context<Self>) {
        let ids = {
            let inbox = self.feed.borrow();
            if !accepts_actions(inbox.status()) {
                return;
            }
            unseen(inbox.items(), &self.reported_seen)
        };
        if ids.is_empty() {
            return;
        }
        self.reported_seen.extend(ids.iter().cloned());
        let handle = self.handle.clone();
        let job = async move { handle.mark_seen(ids).await };
        let task = bridge::run(
            cx,
            &self.runtime,
            "notifications-seen",
            job,
            |_, result, _| {
                if let Err(error) = result {
                    tracing::debug!(error = %error, "seen receipt not recorded");
                }
            },
        );
        self.keep(task);
    }

    fn act(&mut self, action: Action, cx: &mut Context<Self>) {
        if !self.pending.start(&action) {
            return;
        }
        self.failure = None;
        let handle = self.handle.clone();
        let requested = action.clone();
        let job = async move {
            match requested {
                Action::MarkRead(id) => handle.mark_read(vec![id]).await,
                Action::MarkAllRead => handle.mark_all_read().await,
                Action::LoadOlder => handle.load_older().await,
            }
        };
        let task = bridge::run(
            cx,
            &self.runtime,
            "notifications-action",
            job,
            move |this, result, cx| {
                this.pending.finish(&action);
                if let Err(error) = result {
                    tracing::warn!(error = %error, "notifications action failed");
                    this.failure = Some(text::action_failed(&error));
                }
                cx.notify();
            },
        );
        self.keep(task);
        cx.notify();
    }

    fn dismiss_failure(&mut self, cx: &mut Context<Self>) {
        self.failure = None;
        cx.notify();
    }

    fn render_header(&self, inbox: &Inbox, cx: &Context<Self>) -> AnyElement {
        let palette = *tokens(cx);
        let status = inbox.status();
        let live = accepts_actions(status);
        let any_unread = inbox.unread().count > 0;
        let actions = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::SM)
            .child(
                IconButton::new("notifications-refresh", IconName::Sync, text::REFRESH)
                    .small()
                    .disabled(!live)
                    .on_click(cx.listener(|this, _, _, _| this.handle.refresh())),
            )
            .child(
                Button::new("notifications-mark-all")
                    .label(text::MARK_ALL_READ)
                    .icon(IconName::Check)
                    .small()
                    .loading(self.pending.is_running(&Action::MarkAllRead))
                    .disabled(!live || !any_unread)
                    .on_click(cx.listener(|this, _, _, cx| this.act(Action::MarkAllRead, cx))),
            );
        let title = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::MD)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(type_scale::HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.text.primary)
                    .child(text::INBOX_TITLE),
            )
            .child(actions);
        let feed = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(space::XS)
            .text_size(type_scale::SMALL)
            .text_color(palette.text.muted)
            .child(
                StatusDot::new("notifications-feed", feed_tone(status))
                    .pulse(status == FeedStatus::Connecting),
            )
            .child(text::feed_line(status, Utc::now()));
        div()
            .flex()
            .flex_col()
            .gap(space::XS)
            .child(title)
            .child(
                div()
                    .text_size(type_scale::BODY)
                    .text_color(palette.text.secondary)
                    .child(text::INBOX_DESCRIPTION),
            )
            .child(feed)
            .into_any_element()
    }

    fn render_banners(&self, status: FeedStatus, cx: &Context<Self>) -> Vec<AnyElement> {
        let mut banners = Vec::new();
        if let FeedStatus::Failing { .. } = status {
            banners.push(
                Banner::warning("notifications-failing", text::feed_line(status, Utc::now()))
                    .title(text::FAILING_TITLE)
                    .into_any_element(),
            );
        }
        if let Some(message) = self.failure {
            let view = cx.weak_entity();
            banners.push(
                Banner::error("notifications-action-failed", message)
                    .dismissible(text::DISMISS, move |_, cx| {
                        if view.update(cx, Self::dismiss_failure).is_err() {
                            tracing::trace!("failure dismissed after the inbox closed");
                        }
                    })
                    .into_any_element(),
            );
        }
        banners
    }
}

impl Render for InboxView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (header, banners, body) = {
            let inbox = self.feed.borrow();
            (
                self.render_header(&inbox, cx),
                self.render_banners(inbox.status(), cx),
                self.render_body(&inbox, cx),
            )
        };
        div()
            .id("notifications-inbox")
            .size_full()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(space::XL)
                    .w_full()
                    .max_w(px(PAGE_WIDTH))
                    .p(space::XXL)
                    .child(header)
                    .children(banners)
                    .child(body),
            )
    }
}
