//! Pure presentation rules.
//!
//! What the inbox body shows for the feed, whether the reader's actions are
//! offered, which items still owe a seen receipt, which actions are in flight,
//! and the tone of each status.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::collections::HashSet;
use std::hash::BuildHasher;

use harness_ui::Tone;
use sp_notifications::{Attention, FeedStatus, Notification, NotificationId, UnreadCount};
use sp_slack::SlackStatus;

/// What the inbox screen's body shows for the feed's status and whether the
/// inbox holds any item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxBody {
    Withheld,
    NotServed,
    Connecting,
    Unavailable,
    Empty,
    List,
}

pub const fn inbox_body(status: FeedStatus, has_items: bool) -> InboxBody {
    match status {
        FeedStatus::Withheld => InboxBody::Withheld,
        FeedStatus::NotServed => InboxBody::NotServed,
        _ if has_items => InboxBody::List,
        FeedStatus::Connecting => InboxBody::Connecting,
        FeedStatus::Failing { .. } => InboxBody::Unavailable,
        FeedStatus::Live { .. } => InboxBody::Empty,
    }
}

pub const fn accepts_actions(status: FeedStatus) -> bool {
    !matches!(status, FeedStatus::Withheld | FeedStatus::NotServed)
}

pub const fn shows_badge(status: FeedStatus, unread: UnreadCount) -> bool {
    accepts_actions(status) && unread.count > 0
}

pub const fn feed_tone(status: FeedStatus) -> Tone {
    match status {
        FeedStatus::Withheld | FeedStatus::NotServed => Tone::Neutral,
        FeedStatus::Connecting => Tone::Info,
        FeedStatus::Live { .. } => Tone::Success,
        FeedStatus::Failing { .. } => Tone::Warning,
    }
}

pub const fn slack_tone(status: SlackStatus) -> Tone {
    match status {
        SlackStatus::Withheld | SlackStatus::Quiet => Tone::Neutral,
        SlackStatus::Unknown {
            feed: FeedStatus::Failing { .. },
        } => Tone::Warning,
        SlackStatus::Unknown { .. } => Tone::Info,
        SlackStatus::Active { unread: 0, .. } => Tone::Success,
        SlackStatus::Active { .. } => Tone::Accent,
    }
}

pub const fn needs_reader(item: &Notification) -> bool {
    matches!(item.attention, Attention::Human)
}

pub fn unseen<S: BuildHasher>(
    items: &[Notification],
    reported: &HashSet<NotificationId, S>,
) -> Vec<NotificationId> {
    items
        .iter()
        .filter(|item| item.read_at.is_none() && !reported.contains(&item.id))
        .map(|item| item.id.clone())
        .collect()
}

/// One of the reader's actions on the inbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    MarkRead(NotificationId),
    MarkAllRead,
    LoadOlder,
}

/// The reader's actions still in flight. Marking everything read covers
/// every row while it runs.
#[derive(Debug, Default)]
pub struct Pending {
    all: bool,
    older: bool,
    rows: HashSet<NotificationId>,
}

impl Pending {
    pub fn start(&mut self, action: &Action) -> bool {
        if self.is_running(action) {
            return false;
        }
        match action {
            Action::MarkRead(id) => self.rows.insert(id.clone()),
            Action::MarkAllRead => {
                self.all = true;
                true
            },
            Action::LoadOlder => {
                self.older = true;
                true
            },
        }
    }

    pub fn finish(&mut self, action: &Action) {
        match action {
            Action::MarkRead(id) => {
                self.rows.remove(id);
            },
            Action::MarkAllRead => self.all = false,
            Action::LoadOlder => self.older = false,
        }
    }

    pub fn is_running(&self, action: &Action) -> bool {
        match action {
            Action::MarkRead(id) => self.all || self.rows.contains(id),
            Action::MarkAllRead => self.all,
            Action::LoadOlder => self.older,
        }
    }
}
