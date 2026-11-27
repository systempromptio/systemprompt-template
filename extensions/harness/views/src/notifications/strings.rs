//! The English text of both extensions.
//!
//! The notifications inbox, its titlebar bell and the Slack settings pane, with
//! the copy for every feed status and failure.

use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use systemprompt_device::view::SyncErrorKind;
use systemprompt_harness::members::notifications::{FeedStatus, NotificationsError, UnreadCount};
use systemprompt_harness::members::slack::SlackStatus;

pub const NOTIFICATIONS_NAME: &str = "Notifications";
pub const INBOX_TITLE: &str = "Notifications";
pub const INBOX_DESCRIPTION: &str = "What your instance has told you, newest first.";
pub const REFRESH: &str = "Check for new notifications";
pub const MARK_READ: &str = "Mark read";
pub const MARK_ALL_READ: &str = "Mark all read";
pub const LOAD_OLDER: &str = "Load older";
pub const NO_OLDER: &str = "No older notifications.";
pub const NEEDS_YOU: &str = "Needs you";
pub const DISMISS: &str = "Dismiss";

pub const WITHHELD_TITLE: &str = "Notifications are off";
pub const WITHHELD_BODY: &str = "No instance is connected, or the connected instance does not enable notifications for this app.";
pub const NOT_SERVED_TITLE: &str = "This instance has no notifications";
pub const NOT_SERVED_BODY: &str =
    "The instance serves no notifications feed. Ask your administrator to enable it.";
pub const CONNECTING_TITLE: &str = "Connecting to your instance";
pub const UNAVAILABLE_TITLE: &str = "Notifications could not be loaded";
pub const EMPTY_TITLE: &str = "Nothing here yet";
pub const EMPTY_BODY: &str = "Notifications from your instance appear here.";
pub const FAILING_TITLE: &str = "The feed is not up to date";

pub const SLACK_NAME: &str = "Slack";
pub const SLACK_TITLE: &str = "Slack";
pub const SLACK_DESCRIPTION: &str = "Slack activity your instance relays to you as notifications.";
pub const SLACK_HOW_TITLE: &str = "How this works";
pub const SLACK_RELAYED: &str = "Your instance receives Slack activity and turns what concerns you into notifications. Those appear here and in your inbox.";
pub const SLACK_NO_TOKEN: &str = "This app holds no Slack token and never contacts Slack: it cannot post, reply or read channels.";
pub const SLACK_STATUS_TITLE: &str = "Status";
pub const SLACK_ITEMS_TITLE: &str = "From Slack";
pub const SLACK_NO_ITEMS: &str = "No Slack activity in your recent notifications.";

const JUST_NOW: &str = "just now";
const BADGE_MAX: u64 = 99;

pub fn badge(unread: UnreadCount) -> String {
    if unread.count > BADGE_MAX {
        format!("{BADGE_MAX}+")
    } else if unread.capped {
        format!("{}+", unread.count)
    } else {
        unread.count.to_string()
    }
}

pub fn bell_tooltip(unread: UnreadCount) -> String {
    if unread.count == 0 {
        return INBOX_TITLE.to_owned();
    }
    let more = if unread.capped { "+" } else { "" };
    format!("{INBOX_TITLE}, {}{more} unread", unread.count)
}

pub fn feed_line(status: FeedStatus, now: DateTime<Utc>) -> String {
    match status {
        FeedStatus::Withheld => WITHHELD_TITLE.to_owned(),
        FeedStatus::Connecting => CONNECTING_TITLE.to_owned(),
        FeedStatus::Live { refreshed_at } => format!("Updated {}", relative(refreshed_at, now)),
        FeedStatus::NotServed => NOT_SERVED_TITLE.to_owned(),
        FeedStatus::Failing { kind, retry_in } => {
            format!("{} {}", failure(kind), retry(retry_in))
        },
    }
}

pub const fn failure(kind: SyncErrorKind) -> &'static str {
    match kind {
        SyncErrorKind::Unreachable => "The instance could not be reached.",
        SyncErrorKind::RateLimited => "The instance asked this app to slow down.",
        SyncErrorKind::Refused => "The instance refused this app's sign-in. Sign in again.",
        SyncErrorKind::KeyChanged => "The instance's signing key changed.",
        SyncErrorKind::Invalid => "The instance sent an answer this app cannot read.",
        SyncErrorKind::Insecure => "The sign-in could not be opened from secure storage.",
        SyncErrorKind::Service => "The notifications feed failed on the instance.",
    }
}

pub fn retry(retry_in: Duration) -> String {
    let seconds = retry_in.as_secs();
    if seconds < 60 {
        format!("Trying again in {} s.", seconds.max(1))
    } else {
        format!("Trying again in {} min.", seconds.div_ceil(60))
    }
}

pub fn action_failed(error: &NotificationsError) -> &'static str {
    match error {
        NotificationsError::Withheld => WITHHELD_TITLE,
        NotificationsError::NotServed { .. } => NOT_SERVED_TITLE,
        other => failure(other.kind()),
    }
}

pub fn relative(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let elapsed = now.signed_duration_since(at);
    let minutes = elapsed.num_minutes();
    if minutes < 1 {
        JUST_NOW.to_owned()
    } else if minutes < 60 {
        format!("{minutes} min ago")
    } else if elapsed.num_hours() < 24 {
        format!("{} h ago", elapsed.num_hours())
    } else if elapsed.num_days() < 7 {
        format!("{} d ago", elapsed.num_days())
    } else {
        at.with_timezone(&Local).format("%-d %b %Y").to_string()
    }
}

pub fn coalesced(count: i32) -> Option<String> {
    (count > 1).then(|| format!("{count} updates"))
}

pub const fn slack_state(status: SlackStatus) -> &'static str {
    match status {
        SlackStatus::Withheld => "Off",
        SlackStatus::Unknown { .. } => "Not known yet",
        SlackStatus::Quiet => "Quiet",
        SlackStatus::Active { .. } => "Active",
    }
}

pub fn slack_detail(status: SlackStatus, now: DateTime<Utc>) -> String {
    match status {
        SlackStatus::Withheld => {
            "Your instance does not enable Slack and notifications for this app.".to_owned()
        },
        SlackStatus::Unknown { feed } => format!(
            "Slack activity shows once your notifications are live. {}",
            feed_line(feed, now)
        ),
        SlackStatus::Quiet => "Your recent notifications hold nothing from Slack.".to_owned(),
        SlackStatus::Active {
            items,
            unread,
            latest_at,
        } => {
            let noun = if items == 1 { "item" } else { "items" };
            format!(
                "{items} recent {noun}, {unread} unread. Latest {}.",
                relative(latest_at, now)
            )
        },
    }
}
