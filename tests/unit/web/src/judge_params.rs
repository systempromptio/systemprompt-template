//! Scheduler parameter contracts for the conversation judge.

use std::collections::HashMap;

use systemprompt::identifiers::{Actor, UserId};
use systemprompt::traits::JobContext;
use systemprompt_web_extension::jobs::internals::JudgeParams;

fn context(parameters: &[(&str, &str)]) -> JobContext {
    JobContext::new(
        Actor::user(UserId::new("judge-parameter-test")),
        systemprompt::traits::Dependencies::new(),
    )
    .with_parameters(
        parameters
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect::<HashMap<_, _>>(),
    )
}

#[test]
fn judge_parameters_preserve_operational_overrides_and_clamp_resource_limits() {
    let context = context(&[
        ("provider", "local-inference"),
        ("model", "governed-model"),
        ("daily_cost_cap_microdollars", "0"),
        ("context_id", "00000000-0000-4000-8000-00000000a11c"),
        ("batch_size", "999"),
        ("quiet_minutes", "999999"),
        ("lookback_days", "0"),
        ("transcript_token_budget", "999999"),
        ("max_output_tokens", "1"),
    ]);

    let parsed = JudgeParams::from_context(&context).expect("parse scheduler parameters");

    assert_eq!(parsed.provider, "local-inference");
    assert_eq!(parsed.model, "governed-model");
    assert_eq!(parsed.daily_cost_cap_microdollars, 0);
    assert_eq!(
        parsed.context_id.expect("manual context").as_str(),
        "00000000-0000-4000-8000-00000000a11c"
    );
    assert_eq!(parsed.batch_size, 100);
    assert_eq!(parsed.quiet_minutes, 525_600);
    assert_eq!(parsed.lookback_days, 1);
    assert_eq!(parsed.transcript_token_budget, 400_000);
    assert_eq!(parsed.max_output_tokens, 256);
}

#[test]
fn judge_parameters_raise_lower_limits_before_discovery_can_overconsume() {
    let context = context(&[
        ("batch_size", "-1"),
        ("quiet_minutes", "-1"),
        ("lookback_days", "-1"),
        ("transcript_token_budget", "0"),
        ("max_output_tokens", "0"),
    ]);

    let parsed = JudgeParams::from_context(&context).expect("parse bounded scheduler parameters");

    assert_eq!(parsed.batch_size, 1);
    assert_eq!(parsed.quiet_minutes, 0);
    assert_eq!(parsed.lookback_days, 1);
    assert_eq!(parsed.transcript_token_budget, 1_000);
    assert_eq!(parsed.max_output_tokens, 256);
}

#[test]
fn judge_parameters_reject_malformed_numbers_and_manual_context_ids() {
    let malformed = JudgeParams::from_context(&context(&[("batch_size", "many")]));
    assert!(
        malformed
            .expect_err("malformed batch size fails before a judge run")
            .to_string()
            .contains("batch_size=many")
    );

    let invalid_context = JudgeParams::from_context(&context(&[("context_id", "not-a-context")]));
    assert!(
        invalid_context
            .expect_err("manual run must name a valid context")
            .to_string()
            .contains("ContextId")
    );
}
