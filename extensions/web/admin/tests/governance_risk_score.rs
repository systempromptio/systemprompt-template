use systemprompt_web_admin::repositories::governance_grp::risk_score::{
    compute_risk_score, RiskScoreWeights, ViolationCounts,
};

#[test]
fn defaults_match_yaml() {
    let w = RiskScoreWeights::default();
    assert!((w.deny_weight - 1.0).abs() < f64::EPSILON);
    assert!((w.secret_breach_weight - 3.0).abs() < f64::EPSILON);
    assert!((w.scale - 50.0).abs() < f64::EPSILON);
}

#[test]
fn zero_activity_clamps_to_zero() {
    let r = compute_risk_score(&ViolationCounts::default(), RiskScoreWeights::default());
    assert!((r.score - 0.0).abs() < f64::EPSILON);
}

#[test]
fn deny_only_score_within_range() {
    let v = ViolationCounts {
        deny_count: 5,
        secret_breach_count: 0,
        scope_violation_count: 0,
        activity_volume: 50,
    };
    let r = compute_risk_score(&v, RiskScoreWeights::default());
    assert!((r.score - 5.0).abs() < 1e-9);
}

#[test]
fn secret_breach_outweighs_plain_deny() {
    let plain = ViolationCounts {
        deny_count: 1,
        activity_volume: 10,
        ..ViolationCounts::default()
    };
    let breach = ViolationCounts {
        deny_count: 1,
        secret_breach_count: 1,
        activity_volume: 10,
        ..ViolationCounts::default()
    };
    let w = RiskScoreWeights::default();
    assert!(compute_risk_score(&breach, w).score > compute_risk_score(&plain, w).score);
}

#[test]
fn score_clamps_at_100() {
    let v = ViolationCounts {
        deny_count: 1_000_000,
        secret_breach_count: 1_000_000,
        scope_violation_count: 1_000_000,
        activity_volume: 1,
    };
    let r = compute_risk_score(&v, RiskScoreWeights::default());
    assert!((r.score - 100.0).abs() < f64::EPSILON);
}
