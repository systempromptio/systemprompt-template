use std::time::Duration;

use chrono::TimeDelta;
use harness_notifications_ui::strings;
use sp_notifications::{FeedStatus, NotificationsError, UnreadCount};
use sp_protocol::view::SyncErrorKind;
use sp_slack::SlackStatus;

use crate::support::at;

const fn unread(count: u64, capped: bool) -> UnreadCount {
    UnreadCount { count, capped }
}

#[test]
fn the_badge_caps_large_and_uncounted_totals() {
    assert_eq!(strings::badge(unread(5, false)), "5");
    assert_eq!(strings::badge(unread(150, false)), "99+");
    assert_eq!(strings::badge(unread(40, true)), "40+");
}

#[test]
fn the_bell_tooltip_names_the_unread_count() {
    assert_eq!(
        strings::bell_tooltip(unread(0, false)),
        strings::INBOX_TITLE
    );
    assert_eq!(
        strings::bell_tooltip(unread(3, false)),
        "Notifications, 3 unread"
    );
    assert_eq!(
        strings::bell_tooltip(unread(3, true)),
        "Notifications, 3+ unread"
    );
}

#[test]
fn relative_times_read_naturally() {
    let now = at("2026-11-20T12:00:00Z");
    assert_eq!(
        strings::relative(now - TimeDelta::seconds(30), now),
        "just now"
    );
    assert_eq!(
        strings::relative(now + TimeDelta::minutes(5), now),
        "just now"
    );
    assert_eq!(
        strings::relative(now - TimeDelta::minutes(5), now),
        "5 min ago"
    );
    assert_eq!(strings::relative(now - TimeDelta::hours(3), now), "3 h ago");
    assert_eq!(strings::relative(now - TimeDelta::days(2), now), "2 d ago");
    assert!(strings::relative(now - TimeDelta::days(30), now).contains("2026"));
}

#[test]
fn retries_round_up_to_whole_units() {
    assert_eq!(strings::retry(Duration::ZERO), "Trying again in 1 s.");
    assert_eq!(
        strings::retry(Duration::from_secs(30)),
        "Trying again in 30 s."
    );
    assert_eq!(
        strings::retry(Duration::from_secs(90)),
        "Trying again in 2 min."
    );
}

#[test]
fn a_failing_feed_says_why_and_when_it_retries() {
    let line = strings::feed_line(
        FeedStatus::Failing {
            kind: SyncErrorKind::RateLimited,
            retry_in: Duration::from_secs(20),
        },
        at("2026-11-20T12:00:00Z"),
    );
    assert!(line.starts_with(strings::failure(SyncErrorKind::RateLimited)));
    assert!(line.ends_with("Trying again in 20 s."));
}

#[test]
fn a_live_feed_says_when_it_was_updated() {
    let now = at("2026-11-20T12:00:00Z");
    let line = strings::feed_line(
        FeedStatus::Live {
            refreshed_at: now - TimeDelta::minutes(2),
        },
        now,
    );
    assert_eq!(line, "Updated 2 min ago");
    assert_eq!(
        strings::feed_line(FeedStatus::Withheld, now),
        strings::WITHHELD_TITLE
    );
    assert_eq!(
        strings::feed_line(FeedStatus::NotServed, now),
        strings::NOT_SERVED_TITLE
    );
}

#[test]
fn action_failures_name_the_cause() {
    assert_eq!(
        strings::action_failed(&NotificationsError::Withheld),
        strings::WITHHELD_TITLE
    );
    assert_eq!(
        strings::action_failed(&NotificationsError::NotServed {
            endpoint: "GET /feed"
        }),
        strings::NOT_SERVED_TITLE
    );
    assert_eq!(
        strings::action_failed(&NotificationsError::Rejected {
            endpoint: "POST /read",
            status: 400
        }),
        strings::failure(SyncErrorKind::Invalid)
    );
}

#[test]
fn coalesced_counts_show_only_above_one() {
    assert_eq!(strings::coalesced(1), None);
    assert_eq!(strings::coalesced(4).as_deref(), Some("4 updates"));
}

#[test]
fn slack_states_are_stated_plainly() {
    let now = at("2026-11-20T12:00:00Z");
    assert_eq!(strings::slack_state(SlackStatus::Withheld), "Off");
    assert!(strings::slack_detail(SlackStatus::Withheld, now).contains("does not enable Slack"));
    assert_eq!(strings::slack_state(SlackStatus::Quiet), "Quiet");
    let one = SlackStatus::Active {
        items: 1,
        unread: 1,
        latest_at: now - TimeDelta::minutes(3),
    };
    assert_eq!(strings::slack_state(one), "Active");
    assert_eq!(
        strings::slack_detail(one, now),
        "1 recent item, 1 unread. Latest 3 min ago."
    );
    let unknown = strings::slack_detail(
        SlackStatus::Unknown {
            feed: FeedStatus::Connecting,
        },
        now,
    );
    assert!(unknown.ends_with(strings::CONNECTING_TITLE));
}

#[test]
fn the_slack_pane_disclaims_a_token_and_sending() {
    assert!(strings::SLACK_NO_TOKEN.contains("no Slack token"));
    assert!(strings::SLACK_NO_TOKEN.contains("never contacts Slack"));
}
