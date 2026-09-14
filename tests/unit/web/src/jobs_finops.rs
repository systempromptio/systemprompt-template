//! The pure half of the usage-anomaly job: the spike threshold.
//!
//! The I/O half (the queries, the Slack post) is covered by the integration
//! and e2e tiers; what belongs here is the arithmetic — the part where a wrong
//! answer is silent.

use systemprompt_web_extension::jobs::internals::{Finding, evaluate};

#[test]
fn anomaly_needs_both_the_multiplier_and_the_floor() {
    // Past the multiplier but under the floor: a quiet instance stays quiet.
    assert!(evaluate("requests", 30, 5, 3, 50).is_none());
    // Past the floor but under the multiplier: ordinary growth is not a spike.
    assert!(evaluate("requests", 120, 100, 3, 50).is_none());
    // Past both: a finding, carrying what was observed against what was normal.
    let f: Finding = evaluate("requests", 300, 100, 3, 50).expect("a spike");
    assert_eq!((f.metric, f.observed, f.baseline), ("requests", 300, 100));
    // Zero baseline (a brand-new instance): the floor alone decides.
    assert!(evaluate("errors", 9, 0, 5, 10).is_none());
    assert!(evaluate("errors", 10, 0, 5, 10).is_some());
}
