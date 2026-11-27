//! The reader's notifications and Slack, as two shell extensions.
//!
//! [`NotificationsShell`] (id `notifications`) adds the inbox screen, entered
//! from the titlebar: the reader's notifications newest first, mark read, mark
//! all read, load older, and the state of the feed, including a withheld,
//! unserved or failing one. [`unread_bell`] is the titlebar mark with the
//! unread count. [`SlackShell`] (id `slack`, which depends on `notifications`)
//! adds a settings pane that says what Slack is here: activity the instance
//! relays as notifications, never a Slack connection of this app.
//!
//! Every view renders from its sp handle's own feed and keeps no copy of the
//! inbox. The shell shows these contributions only while the instance admits
//! the extension.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod bell;
mod bridge;
mod extension;
mod inbox_view;
/// Pure presentation rules over the feed, the inbox and Slack's status.
pub mod model;
mod slack_pane;
/// The English text of both extensions.
pub mod strings;

pub use bell::unread_bell;
pub use extension::{INBOX_SCREEN, NotificationsShell, SLACK_PANE, SlackShell};
