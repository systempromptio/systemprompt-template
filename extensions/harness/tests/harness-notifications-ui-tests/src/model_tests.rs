use std::collections::HashSet;
use std::time::Duration;

use harness_notifications_ui::model::{
    Action, InboxBody, Pending, accepts_actions, feed_tone, inbox_body, needs_reader, shows_badge,
    slack_tone, unseen,
};
use harness_ui::Tone;
use sp_notifications::{FeedStatus, UnreadCount};
use sp_protocol::view::SyncErrorKind;
use sp_slack::SlackStatus;

use crate::support::{at, id_of, notification};

const FAILING: FeedStatus = FeedStatus::Failing {
    kind: SyncErrorKind::Unreachable,
    retry_in: Duration::from_secs(30),
};

fn live() -> FeedStatus {
    FeedStatus::Live {
        refreshed_at: at("2026-11-20T10:00:00Z"),
    }
}

#[test]
fn withheld_and_unserved_feeds_show_their_state_even_with_items() {
    assert_eq!(inbox_body(FeedStatus::Withheld, true), InboxBody::Withheld);
    assert_eq!(inbox_body(FeedStatus::Withheld, false), InboxBody::Withheld);
    assert_eq!(
        inbox_body(FeedStatus::NotServed, true),
        InboxBody::NotServed
    );
}

#[test]
fn held_items_stay_listed_while_the_feed_connects_or_fails() {
    assert_eq!(inbox_body(FeedStatus::Connecting, true), InboxBody::List);
    assert_eq!(inbox_body(FAILING, true), InboxBody::List);
    assert_eq!(inbox_body(live(), true), InboxBody::List);
}

#[test]
fn an_empty_inbox_says_why_it_is_empty() {
    assert_eq!(
        inbox_body(FeedStatus::Connecting, false),
        InboxBody::Connecting
    );
    assert_eq!(inbox_body(FAILING, false), InboxBody::Unavailable);
    assert_eq!(inbox_body(live(), false), InboxBody::Empty);
}

#[test]
fn actions_are_refused_while_withheld_or_unserved() {
    assert!(!accepts_actions(FeedStatus::Withheld));
    assert!(!accepts_actions(FeedStatus::NotServed));
    assert!(accepts_actions(FeedStatus::Connecting));
    assert!(accepts_actions(FAILING));
    assert!(accepts_actions(live()));
}

#[test]
fn the_badge_shows_only_unread_items_of_an_admitted_feed() {
    let three = UnreadCount {
        count: 3,
        capped: false,
    };
    assert!(shows_badge(live(), three));
    assert!(shows_badge(FAILING, three));
    assert!(!shows_badge(FeedStatus::Withheld, three));
    assert!(!shows_badge(FeedStatus::NotServed, three));
    assert!(!shows_badge(live(), UnreadCount::default()));
}

#[test]
fn feed_tones_follow_the_status() {
    assert_eq!(feed_tone(FeedStatus::Withheld), Tone::Neutral);
    assert_eq!(feed_tone(FeedStatus::Connecting), Tone::Info);
    assert_eq!(feed_tone(live()), Tone::Success);
    assert_eq!(feed_tone(FAILING), Tone::Warning);
}

#[test]
fn slack_tones_follow_the_status() {
    assert_eq!(slack_tone(SlackStatus::Withheld), Tone::Neutral);
    assert_eq!(slack_tone(SlackStatus::Quiet), Tone::Neutral);
    assert_eq!(
        slack_tone(SlackStatus::Unknown { feed: FAILING }),
        Tone::Warning
    );
    assert_eq!(
        slack_tone(SlackStatus::Unknown {
            feed: FeedStatus::Connecting
        }),
        Tone::Info
    );
    let latest_at = at("2026-11-20T10:00:00Z");
    assert_eq!(
        slack_tone(SlackStatus::Active {
            items: 2,
            unread: 0,
            latest_at
        }),
        Tone::Success
    );
    assert_eq!(
        slack_tone(SlackStatus::Active {
            items: 2,
            unread: 1,
            latest_at
        }),
        Tone::Accent
    );
}

#[test]
fn only_human_attention_needs_the_reader() {
    assert!(needs_reader(&notification(0, false, "human")));
    assert!(!needs_reader(&notification(0, false, "agent")));
    assert!(!needs_reader(&notification(0, false, "silent")));
}

#[test]
fn unseen_lists_unread_items_not_yet_reported() {
    let items = vec![
        notification(0, false, "human"),
        notification(1, true, "human"),
        notification(2, false, "human"),
    ];
    assert_eq!(unseen(&items, &HashSet::new()), vec![id_of(0), id_of(2)]);
    let reported: HashSet<_> = std::iter::once(id_of(0)).collect();
    assert_eq!(unseen(&items, &reported), vec![id_of(2)]);
}

#[test]
fn an_action_in_flight_is_not_started_twice() {
    let mut pending = Pending::default();
    let read = Action::MarkRead(id_of(1));
    assert!(pending.start(&read));
    assert!(!pending.start(&read));
    assert!(pending.is_running(&read));
    assert!(!pending.is_running(&Action::MarkRead(id_of(2))));
    pending.finish(&read);
    assert!(!pending.is_running(&read));
    assert!(pending.start(&read));
}

#[test]
fn marking_everything_read_covers_every_row() {
    let mut pending = Pending::default();
    assert!(pending.start(&Action::MarkAllRead));
    assert!(pending.is_running(&Action::MarkRead(id_of(7))));
    assert!(!pending.start(&Action::MarkRead(id_of(7))));
    assert!(!pending.is_running(&Action::LoadOlder));
    pending.finish(&Action::MarkAllRead);
    assert!(!pending.is_running(&Action::MarkRead(id_of(7))));
}
