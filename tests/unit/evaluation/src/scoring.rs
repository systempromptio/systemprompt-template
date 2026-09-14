use std::collections::{BTreeMap, BTreeSet};
use systemprompt_evaluation::experiments::content_digest;
use systemprompt_evaluation::experiments::resources::{
    CaseContent, Partition, ResourceContent, RubricContent, WeightedDimension,
};
use systemprompt_evaluation::experiments::scoring::{EvidenceJudgment, EvidenceScore, score};

fn rubric() -> RubricContent {
    RubricContent {
        dimensions: vec![
            WeightedDimension {
                name: "grounding".into(),
                description: "Claims are supported by cited evidence".into(),
                weight: 3,
            },
            WeightedDimension {
                name: "clarity".into(),
                description: "Reader can identify scope and decisions".into(),
                weight: 1,
            },
        ],
        pass_threshold_milli: 4000,
        hard_gates: vec!["approval".into()],
    }
}

fn judgment() -> EvidenceJudgment {
    EvidenceJudgment {
        dimensions: vec![
            EvidenceScore {
                name: "grounding".into(),
                score: 5,
                evidence: vec!["source-1".into()],
            },
            EvidenceScore {
                name: "clarity".into(),
                score: 1,
                evidence: vec!["source-1".into()],
            },
        ],
        hard_gates: BTreeMap::from([("approval".into(), true)]),
        rationale: "Grounded output with difficult prose".into(),
    }
}

#[test]
fn weighted_score_is_computed_from_dimensions_and_hard_gates() {
    let evidence = BTreeSet::from(["source-1".into()]);
    let result = score(&rubric(), &judgment(), &evidence).unwrap();
    assert_eq!(result.score_milli, 4000);
    assert!(result.passed);
    let mut rejected = judgment();
    rejected.hard_gates.insert("approval".into(), false);
    assert!(!score(&rubric(), &rejected, &evidence).unwrap().passed);
}

#[test]
fn incomplete_or_fabricated_judgments_cannot_receive_a_score() {
    let evidence = BTreeSet::from(["source-1".into()]);
    let mut missing = judgment();
    missing.dimensions.pop();
    assert!(score(&rubric(), &missing, &evidence).is_err());
    let mut duplicate = judgment();
    duplicate.dimensions[1].name = "grounding".into();
    assert!(score(&rubric(), &duplicate, &evidence).is_err());
    let mut invented = judgment();
    invented.dimensions[0].evidence = vec!["fabricated-source".into()];
    assert!(score(&rubric(), &invented, &evidence).is_err());
    let mut out_of_range = judgment();
    out_of_range.dimensions[0].score = 6;
    assert!(score(&rubric(), &out_of_range, &evidence).is_err());
    let mut missing_gate = judgment();
    missing_gate.hard_gates.clear();
    assert!(score(&rubric(), &missing_gate, &evidence).is_err());
}

#[test]
fn fixture_paths_cannot_escape_execution_workspace() {
    for path in [
        "../credential",
        "/etc/passwd",
        "folder/../../secret",
        "folder\\secret",
        "folder//file",
    ] {
        let case = ResourceContent::Case(CaseContent {
            prompt: "Write a specification".into(),
            expected_behavior: vec!["Cite sources".into()],
            assertions: Vec::new(),
            fixtures: BTreeMap::from([(path.into(), "fixture".into())]),
            partition: Partition::Development,
        });
        assert!(case.validate().is_err(), "{path}");
    }
}

#[test]
fn content_digest_is_canonical_and_sensitive_to_reference_changes() {
    assert_eq!(
        content_digest(&serde_json::json!({"a":1,"b":2})).unwrap(),
        content_digest(&serde_json::json!({"b":2,"a":1})).unwrap()
    );
    assert_ne!(
        content_digest(&serde_json::json!({"reference":"v1"})).unwrap(),
        content_digest(&serde_json::json!({"reference":"v2"})).unwrap()
    );
}

#[test]
fn experiment_matrix_rejects_duplicate_variants_and_accepts_repetitions() {
    use systemprompt_evaluation::experiments::ExperimentSpec;
    let variant = serde_json::json!({
        "client":"claude-code", "client_version":"pinned", "model":"haiku",
        "provider":"anthropic", "skill_bundle_digest":"a".repeat(64),
        "configuration_digest":"b".repeat(64), "worker_image_digest":"c".repeat(64)
    });
    let mut spec: ExperimentSpec = serde_json::from_value(serde_json::json!({
        "schema_version":1, "name":"matrix", "cases":["case-1"], "rubric":"rubric-1",
        "variants":[variant.clone(),variant], "repetitions":1,
        "budget_microdollars":5_000_000, "execution_mode":"fixture", "objective":"quality"
    }))
    .unwrap();
    assert!(spec.validate().is_err());
    spec.variants.pop();
    spec.repetitions = 2;
    assert!(spec.validate().is_ok());
}
