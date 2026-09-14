//! The overview's pure rules: when an MCP server counts as alive, how a
//! window-over-window delta is derived, and which colour an anomaly earns.
//!
//! Both decide what an operator sees on the console landing page before they
//! have clicked anything, so both are pinned here rather than left to the one
//! integration run that happens to have a heartbeat in the right place.

use chrono::{DateTime, Duration, Utc};

use systemprompt_web_admin::repositories::overview::kpis::OverviewKpis;
use systemprompt_web_admin::repositories::overview::liveness::{
    HEARTBEAT_INTERVAL_SECS, Liveness, liveness_state,
};
use systemprompt_web_admin::repositories::overview::queues::anomaly_tone;
use systemprompt_web_admin::util::delta::{DeltaKind, delta};

fn now() -> DateTime<Utc> {
    DateTime::from_timestamp(1_756_000_000, 0).expect("fixed timestamp")
}

#[test]
fn a_server_no_session_has_ever_touched_is_silent_not_dead() {
    let state = liveness_state(now(), None, HEARTBEAT_INTERVAL_SECS);
    assert_eq!(state, Liveness::Silent);
    assert_eq!(state.label(), "No sessions");
    assert!(!state.is_alive());
}

// Why: the rule is two intervals, and the boundary is inclusive. A server that
// beat exactly one interval late is one missed beat, which is a slow response
// rather than a failure.
#[test]
fn one_missed_beat_is_still_alive() {
    let last = now() - Duration::seconds(HEARTBEAT_INTERVAL_SECS);
    assert_eq!(
        liveness_state(now(), Some(last), HEARTBEAT_INTERVAL_SECS),
        Liveness::Alive
    );
}

#[test]
fn the_boundary_is_two_intervals_inclusive() {
    let on_edge = now() - Duration::seconds(HEARTBEAT_INTERVAL_SECS * 2);
    let past_edge = now() - Duration::seconds(HEARTBEAT_INTERVAL_SECS * 2 + 1);
    assert_eq!(
        liveness_state(now(), Some(on_edge), HEARTBEAT_INTERVAL_SECS),
        Liveness::Alive
    );
    assert_eq!(
        liveness_state(now(), Some(past_edge), HEARTBEAT_INTERVAL_SECS),
        Liveness::Stale
    );
}

// Why: clock skew between the server writing the row and the console reading
// it is not evidence of a problem, so a beat from the future reads as alive
// rather than as an impossible negative age.
#[test]
fn a_heartbeat_from_the_future_is_alive() {
    let ahead = now() + Duration::seconds(30);
    assert_eq!(
        liveness_state(now(), Some(ahead), HEARTBEAT_INTERVAL_SECS),
        Liveness::Alive
    );
}

#[test]
fn every_state_paints_a_distinct_tone() {
    assert_eq!(Liveness::Alive.tone(), "ok");
    assert_eq!(Liveness::Stale.tone(), "warn");
    assert_eq!(Liveness::Silent.tone(), "muted");
}

// Why: the polarity is the caller's, not the sign's. The same rise is good on
// the requests tile and bad on the spend tile, and the tone is what a reader
// takes the colour from.
#[test]
fn the_same_rise_is_good_on_requests_and_bad_on_spend() {
    let requests = delta(120, 100, true);
    let spend = delta(120, 100, false);
    assert_eq!(requests.direction, "up");
    assert_eq!(spend.direction, "up");
    assert_eq!(requests.tone, "good");
    assert_eq!(spend.tone, "bad");
    assert_eq!(requests.display(), "+20.0%");
}

#[test]
fn a_window_with_no_predecessor_reads_as_new_and_an_empty_pair_as_nothing() {
    assert_eq!(delta(40, 0, true).display_kind, DeltaKind::New);
    assert_eq!(delta(0, 0, true).display_kind, DeltaKind::None);
    assert_eq!(delta(0, 0, true).display(), "\u{2014}");
    assert_eq!(delta(100, 100, true).direction, "flat");
}

#[test]
fn a_fall_carries_a_minus_sign_rather_than_a_hyphen() {
    assert_eq!(delta(50, 100, true).display(), "\u{2212}50.0%");
}

// Why: the error rate is carried as tenths of a percent so the tile and the
// delta are derived from one integer; two floats would let them round apart.
#[test]
fn the_error_rate_is_tenths_of_a_percent_of_the_window() {
    let kpis = OverviewKpis {
        requests: 240,
        errors: 24,
        prev_requests: 100,
        prev_errors: 20,
        ..OverviewKpis::default()
    };
    assert_eq!(kpis.error_rate_tenths(), 100);
    assert_eq!(kpis.prev_error_rate_tenths(), 200);
    assert_eq!(
        delta(
            kpis.error_rate_tenths(),
            kpis.prev_error_rate_tenths(),
            false
        )
        .tone,
        "good"
    );
}

// Why: an empty window has no rate. Zero would claim a measurement nobody
// took, and the tile renders the em dash off exactly this case.
#[test]
fn an_empty_window_has_no_error_rate() {
    assert_eq!(OverviewKpis::default().error_rate_tenths(), 0);
    assert_eq!(OverviewKpis::default().requests, 0);
}

// Why: the badge is the only place the table says how bad a row is, so the
// bands are pinned: red at twice the baseline, amber from one and a half, and
// a metric with nothing to compare against is a caution rather than a spike.
#[test]
fn anomaly_severity_bands() {
    assert_eq!(anomaly_tone(60, 0), "warn");
    assert_eq!(anomaly_tone(200, 100), "err");
    assert_eq!(anomaly_tone(150, 100), "warn");
    assert_eq!(anomaly_tone(149, 100), "muted");
    assert_eq!(anomaly_tone(0, 100), "muted");
}
