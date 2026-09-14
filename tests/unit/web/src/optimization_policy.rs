//! Optimization evidence and resource-accounting invariants.

use systemprompt::analytics::resource_metrics::{ResourceFact, aggregate};
use systemprompt::evaluation::campaigns::comparison::{Outcome, PairedOutcome, compare};
use systemprompt::evaluation::campaigns::{CampaignPolicy, OptimizationObjective};
use systemprompt::identifiers::{
    AiRequestId, EvalBudgetId, ManagedResourceId, ResourceInvocationId, ResourceRevisionId,
    SessionId, UserId,
};

fn policy() -> CampaignPolicy {
    CampaignPolicy {
        name: "Reduce tokens".to_owned(),
        resource_id: ManagedResourceId::generate(),
        baseline_revision_id: ResourceRevisionId::generate(),
        budget_id: EvalBudgetId::generate(),
        objective: OptimizationObjective::Tokens,
        minimum_quality_milli: 4000,
        minimum_pairs: 10,
        maximum_iterations: 3,
        automatic: true,
    }
}

fn pair() -> PairedOutcome {
    let baseline = Outcome {
        quality_milli: 4500,
        tokens: 1000,
        cost_microdollars: 50,
        latency_ms: 100,
        verified_success: true,
        hard_failures: 0,
        accounting_complete: true,
    };
    let candidate = Outcome {
        tokens: 800,
        ..baseline
    };
    PairedOutcome {
        baseline,
        candidate,
    }
}

#[test]
fn cheaper_candidates_require_quality_accounting_and_sample_evidence() {
    let policy = policy();
    let mut pairs = vec![pair(); 10];
    assert!(compare(&policy, &pairs).unwrap().eligible);
    assert!(!compare(&policy, &pairs[..2]).unwrap().eligible);
    pairs[0].candidate.accounting_complete = false;
    assert!(!compare(&policy, &pairs).unwrap().eligible);
    pairs[0].candidate.accounting_complete = true;
    pairs[0].candidate.hard_failures = 1;
    assert!(!compare(&policy, &pairs).unwrap().eligible);
    pairs[0].candidate.hard_failures = 0;
    pairs[0].candidate.quality_milli = 4000;
    assert!(!compare(&policy, &pairs).unwrap().eligible);
}

#[test]
fn noisy_token_savings_do_not_establish_an_improvement() {
    let mut pairs = vec![pair(); 10];
    pairs[0].candidate.tokens = 2790;
    let decision = compare(&policy(), &pairs).unwrap();
    assert!(decision.mean_improvement.unwrap() > 0.0);
    assert!(!decision.eligible);
}

#[test]
fn repetitions_do_not_create_independent_samples_or_hide_hard_failures() {
    use systemprompt::evaluation::campaigns::comparison::collapse_repetitions;
    let mut repetitions = vec![pair(); 100];
    let collapsed = collapse_repetitions(&repetitions).unwrap();
    assert!(!compare(&policy(), &[collapsed]).unwrap().eligible);
    repetitions[99].candidate.hard_failures = 1;
    assert_eq!(
        collapse_repetitions(&repetitions)
            .unwrap()
            .candidate
            .hard_failures,
        1
    );
    assert!(collapse_repetitions(&[]).is_err());
}

fn fact() -> ResourceFact {
    ResourceFact {
        invocation_id: ResourceInvocationId::new("invocation-1"),
        user_id: UserId::new("consumer"),
        session_id: SessionId::new("session"),
        invoked_at: chrono::Utc::now(),
        request_id: Some(AiRequestId::new("request-1")),
        input_tokens: Some(100),
        output_tokens: Some(20),
        cache_read_tokens: Some(30),
        cache_creation_tokens: Some(10),
        cost_microdollars: Some(70),
        latency_ms: Some(500),
        failed: true,
        quality_score: Some(3.0),
        successful: Some(false),
        revision_verified: false,
    }
}

#[test]
fn requests_and_assessments_are_deduplicated_and_failed_spend_is_retained() {
    let first = fact();
    let second = ResourceFact {
        invocation_id: ResourceInvocationId::new("invocation-2"),
        ..first.clone()
    };
    let third = ResourceFact {
        request_id: Some(AiRequestId::new("request-2")),
        cost_microdollars: None,
        input_tokens: None,
        output_tokens: None,
        ..first.clone()
    };
    let result = aggregate([&first, &first, &second, &third]);
    assert_eq!(result.invocations, 2);
    assert_eq!(result.requests, 2);
    assert_eq!(result.measured_requests, 1);
    assert_eq!(result.priced_requests, 1);
    assert_eq!(result.related_cost_microdollars, Some(70));
    assert_eq!(result.average_tokens_per_measured_request, Some(160.0));
    assert_eq!(result.assessed_conversations, 1);
    assert_eq!(result.average_quality_score, Some(3.0));
}

#[test]
fn missing_cost_is_distinct_from_measured_zero_cost() {
    let empty = aggregate(std::iter::empty());
    assert_eq!(empty.related_cost_microdollars, None);
    assert_eq!(empty.average_tokens_per_measured_request, None);
    let free = ResourceFact {
        cost_microdollars: Some(0),
        ..fact()
    };
    assert_eq!(aggregate([&free]).related_cost_microdollars, Some(0));
}

#[test]
fn session_quality_is_not_weighted_by_request_count() {
    let first = fact();
    let second = ResourceFact {
        request_id: Some(AiRequestId::new("second")),
        ..first.clone()
    };
    let third = ResourceFact {
        request_id: Some(AiRequestId::new("third")),
        session_id: SessionId::new("other-session"),
        quality_score: Some(5.0),
        ..first.clone()
    };
    assert_eq!(
        aggregate([&first, &second, &third]).average_quality_score,
        Some(4.0)
    );
}
