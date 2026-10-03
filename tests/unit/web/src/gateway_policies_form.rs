//! The policy editor's form pairs → a spec core would accept, with every
//! refusal in the operator's words.

use systemprompt::gateway::{QuotaMode, SafetyHistoryMode, SafetyMode};
use systemprompt_web_admin::repositories::gateway_policies::form::parse_policy_form;
use systemprompt_web_admin::repositories::gateway_policies::month_window::MONTH_WINDOW_SECONDS;

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn full_form() -> Vec<(String, String)> {
    pairs(&[
        ("name", "default-quotas"),
        ("enabled", "true"),
        ("priority", "0"),
        ("quota_mode", "warn"),
        ("w0_subject", "user"),
        ("w0_window", "hour"),
        ("w0_window", ""),
        ("w0_max_requests", "600"),
        ("w0_max_cost", "$20"),
        ("w1_subject", "organization"),
        ("w1_window", "month"),
        ("w1_window", ""),
        ("w1_max_cost", "200.50"),
        ("w2_subject", ""),
        ("w2_window", "day"),
        ("safety_mode", "warn"),
        ("scanner", "heuristic"),
        ("scanner", "pii_extended"),
        ("block", "pii_ssn"),
        ("block_response", "secret"),
        ("block_response", "pii_ssn"),
        ("history", "off"),
        (
            "heuristic_phrases",
            "ignore previous instructions\n\n  act as dan  \n",
        ),
    ])
}

#[test]
fn a_complete_form_becomes_the_spec_the_file_would_declare() {
    let parsed = parse_policy_form(&full_form()).expect("parses");
    assert_eq!(parsed.name, "default-quotas");
    assert!(parsed.enabled);
    let spec = parsed.spec;
    assert_eq!(spec.quota_mode, QuotaMode::Warn);
    assert_eq!(
        spec.quota_windows.len(),
        2,
        "a row with no subject is skipped"
    );
    assert_eq!(spec.quota_windows[0].subject, "user");
    assert_eq!(spec.quota_windows[0].window_seconds, 3_600);
    assert_eq!(spec.quota_windows[0].max_requests, Some(600));
    assert_eq!(
        spec.quota_windows[0].max_cost_microdollars,
        Some(20_000_000)
    );
    assert_eq!(spec.quota_windows[1].window_seconds, MONTH_WINDOW_SECONDS);
    assert_eq!(
        spec.quota_windows[1].max_cost_microdollars,
        Some(200_500_000)
    );
    assert_eq!(spec.safety.mode, SafetyMode::Warn);
    assert_eq!(spec.safety.scanners, vec!["heuristic", "pii_extended"]);
    assert_eq!(spec.safety.block_categories, vec!["pii_ssn"]);
    assert_eq!(
        spec.safety.block_response_categories,
        vec!["secret", "pii_ssn"]
    );
    assert_eq!(spec.safety.history, SafetyHistoryMode::Off);
    assert_eq!(
        spec.safety.heuristic.phrases,
        Some(vec![
            "ignore previous instructions".to_owned(),
            "act as dan".to_owned()
        ])
    );
}

// Why: the period arrives as the select and the custom field behind it,
// both named `w<i>_window`; "custom" is the select deferring to the number.
#[test]
fn a_custom_period_reads_the_seconds_field_behind_the_select() {
    let mut form = full_form();
    form.retain(|(k, _)| k != "w0_window");
    form.push(("w0_window".to_owned(), "custom".to_owned()));
    form.push(("w0_window".to_owned(), "1800".to_owned()));
    let parsed = parse_policy_form(&form).expect("parses");
    assert_eq!(parsed.spec.quota_windows[0].window_seconds, 1_800);
}

#[test]
fn refusals_are_specific() {
    let cases: [(&[(&str, &str)], &str); 6] = [
        (&[("enabled", "true")], "a policy needs a name"),
        (&[("name", "bad name")], "letters, digits"),
        (
            &[("name", "p"), ("w0_subject", "user"), ("w0_window", "hour")],
            "sets no ceiling",
        ),
        (
            &[
                ("name", "p"),
                ("w0_subject", "cost-centre"),
                ("w0_window", "hour"),
            ],
            "not a quota subject",
        ),
        (
            &[
                ("name", "p"),
                ("w0_subject", "user"),
                ("w0_window", "hour"),
                ("w0_max_requests", "-1"),
            ],
            "must not be negative",
        ),
        (&[("name", "p"), ("scanner", "llm_judge")], "not a scanner"),
    ];
    for (fields, expected) in cases {
        let err = parse_policy_form(&pairs(fields))
            .expect_err("refused")
            .to_string();
        assert!(
            err.contains(expected),
            "{err:?} should mention {expected:?}"
        );
    }
}

// Why: an empty phrase box is "use core's builtin list", not "no phrases",
// so the heuristic scanner stays valid — the parser folds the blank to
// `None` and core's validator counts the builtin.
#[test]
fn an_empty_phrase_list_falls_back_to_the_builtin_and_is_accepted() {
    let form = pairs(&[
        ("name", "p"),
        ("scanner", "heuristic"),
        ("heuristic_phrases", ""),
    ]);
    let parsed = parse_policy_form(&form).expect("accepted");
    assert_eq!(parsed.spec.safety.heuristic.phrases, None);
}
