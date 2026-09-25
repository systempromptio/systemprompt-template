use systemprompt::ai::SafetyScanner;
use systemprompt::identifiers::{CallId, SessionId, UserId};
use systemprompt::models::wire::canonical::{
    CanonicalContent, CanonicalMessage, CanonicalRequest, Role,
};
use systemprompt::models::wire::inspect::{SurfaceBudget, string_leaves};
use systemprompt_security::authz::types::Decision;
use systemprompt_security::policy::secrets::redact_spans;
use systemprompt_security::policy::types::AccessScope;
use systemprompt_security::policy::{
    AgentScope, GovernanceConfig, GovernanceEngine, GovernedInput, GovernedTarget, McpToolInput,
    PolicyContext,
};
use systemprompt_web_admin::gateway_safety::PiiScanner;

fn engine(mode: &str) -> GovernanceEngine {
    GovernanceEngine::from_config(
        &GovernanceConfig::parse(&format!(
            "governance:\n  mode: {mode}\n  policies:\n    - id: secret_scan\n      patterns:\n        - id: github-token-classic\n          name: GitHub Token (classic)\n          regex: '\\bghp_[A-Za-z0-9]{{36,}}'\n"
        ))
        .unwrap(),
    )
    .unwrap()
}

fn context<'a>(
    input: &'a GovernedInput,
    user: &'a UserId,
    session: &'a SessionId,
    call: &'a CallId,
) -> PolicyContext<'a> {
    PolicyContext {
        target: GovernedTarget::Prompt,
        agent_scope: AgentScope::User {
            user_id: user.clone(),
        },
        access_scope: AccessScope::User,
        user_id: user,
        session_id: session,
        call_id: call,
        input,
    }
}

#[test]
fn aws_names_and_references_are_not_credentials() {
    let engine = engine("warn");
    let scanner = engine.secret_scanner().expect("configured scanner");
    for text in [
        "AWS_SECRET_ACCESS_KEY",
        "Do not expose aws_secret_access_key",
        "AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}",
        "aws_secret_access_key =",
        "aws_secret_access_key = sensitive-value",
    ] {
        assert!(
            scanner
                .detect(&GovernedInput::prompt_text(text.to_owned()))
                .is_none(),
            "{text}"
        );
    }
}

#[test]
fn aws_shaped_text_and_structured_values_have_no_confirmed_finding() {
    let engine = GovernanceEngine::from_config(
        &GovernanceConfig::parse(
            "governance:\n  policies:\n    - id: secret_scan\n      entropy:\n        enabled: false\n",
        )
        .unwrap(),
    )
    .unwrap();
    let scanner = engine.secret_scanner().expect("empty scanner exists");
    assert!(
        scanner
            .detect(&GovernedInput::prompt_text(
                "AKIAIOSFODNN7EXAMPLE".to_owned()
            ))
            .is_none()
    );
    let input = GovernedInput::tool_arguments(McpToolInput::new(
        serde_json::json!({"AWS_SECRET_ACCESS_KEY": "Ab9/".repeat(10)}),
    ));
    assert!(scanner.detect(&input).is_none());
}

#[test]
fn entropy_never_blocks_or_invokes_recovery_but_remains_in_evidence() {
    let input =
        GovernedInput::prompt_text("PHL+ERIbxzlQOeiiRybQwgV7GvYmIclsJe1zsFIyuuM".to_owned());
    let user = UserId::new("calibration-user");
    let session = SessionId::generate();
    let call = CallId::generate();
    for mode in ["warn", "enforce"] {
        let result = engine(mode)
            .evaluate_with_prompt_recovery(&context(&input, &user, &session, &call), |_| {
                panic!("entropy must not redact")
            });
        assert!(matches!(result.decision, Decision::Allow { .. }));
        assert!(
            result
                .chain
                .iter()
                .any(|entry| entry.detail.contains("observation: high-entropy-token"))
        );
    }
}

#[test]
fn confirmed_secret_recovery_is_required_on_every_replayed_turn() {
    let token = format!("ghp_{}", "Ab9x".repeat(9));
    let original = format!("keep this {token} and that");
    let input = GovernedInput::prompt_text(original.clone());
    let user = UserId::new("calibration-user");
    let session = SessionId::generate();
    let engine = engine("enforce");
    for _ in 0..2 {
        let call = CallId::generate();
        let ctx = context(&input, &user, &session, &call);
        let repaired = engine.evaluate_with_prompt_recovery(&ctx, |hits| {
            let text = redact_spans(&original, hits.iter().map(|hit| hit.span.clone())).unwrap();
            assert_eq!(text, "keep this [REDACTED_BY_GOVERNANCE] and that");
            Some(GovernedInput::prompt_text(text))
        });
        assert!(repaired.decision.permits());
        let failed = engine.evaluate_with_prompt_recovery(&ctx, |_| None);
        assert!(matches!(failed.decision, Decision::Deny { .. }));
        let ineffective = engine.evaluate_with_prompt_recovery(&ctx, |_| Some(input.clone()));
        assert!(matches!(ineffective.decision, Decision::Deny { .. }));
    }
}

#[test]
fn warn_mode_never_repairs_a_confirmed_secret() {
    let input = GovernedInput::prompt_text(format!("ghp_{}", "Ab9x".repeat(9)));
    let user = UserId::new("calibration-user");
    let session = SessionId::generate();
    let call = CallId::generate();
    let result = engine("warn")
        .evaluate_with_prompt_recovery(&context(&input, &user, &session, &call), |_| {
            panic!("warn must not mutate")
        });
    assert!(matches!(result.decision, Decision::Warn { .. }));
}

#[test]
fn early_entropy_does_not_hide_a_later_confirmed_credential() {
    let input = GovernedInput::prompt_parts(vec![
        (
            "first".to_owned(),
            "PHL+ERIbxzlQOeiiRybQwgV7GvYmIclsJe1zsFIyuuM".to_owned(),
        ),
        ("second".to_owned(), format!("ghp_{}", "Ab9x".repeat(9))),
    ]);
    assert_eq!(
        engine("warn")
            .secret_scanner()
            .unwrap()
            .detect(&input)
            .unwrap()
            .pattern
            .id,
        "github-token-classic"
    );
}

#[tokio::test]
async fn historical_pii_is_separate_in_canonical_and_forwarded_requests() {
    let mut request = CanonicalRequest {
        messages: vec![
            CanonicalMessage {
                role: Role::User,
                content: vec![CanonicalContent::text("Phone +442079460100".to_owned())],
            },
            CanonicalMessage {
                role: Role::User,
                content: vec![CanonicalContent::text("Summarize".to_owned())],
            },
        ],
        ..Default::default()
    };
    for forwarded in [false, true] {
        if forwarded {
            request.forwarded_surface = string_leaves(br#"{"messages":[{"role":"user","content":"Phone +442079460100"},{"role":"user","content":"Summarize"}]}"#, SurfaceBudget::default());
        }
        assert!(PiiScanner::new().scan_request(&request).await.is_empty());
        let history = PiiScanner::new().scan_request_history(&request).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].phase, "request_history");
    }
}

#[test]
fn installation_omits_aws_and_keeps_warn_mode() {
    let cfg = GovernanceConfig::parse(include_str!("../../../../services/governance/config.yaml"))
        .unwrap();
    assert_eq!(cfg.mode.to_string(), "warn");
    assert!(
        cfg.policies
            .iter()
            .all(|policy| policy.mode.to_string() == "warn")
    );
    let engine = GovernanceEngine::from_config(&cfg).unwrap();
    assert!(!engine.enforces_prompt_secrets());
    let secret_scan = cfg
        .policies
        .iter()
        .find(|policy| policy.id == "secret_scan")
        .unwrap();
    let ids = secret_scan.params["patterns"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|pattern| pattern["id"].as_str())
        .collect::<Vec<_>>();
    assert!(ids.iter().all(|id| !id.starts_with("aws-")));
}
