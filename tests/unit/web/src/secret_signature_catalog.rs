use systemprompt_security::policy::governed::{GovernedInput, McpToolInput};
use systemprompt_security::policy::{GovernanceConfig, GovernanceEngine};

fn shipped_engine() -> GovernanceEngine {
    let path = crate::support::repo_root().join("services/governance/config.yaml");
    let yaml = std::fs::read_to_string(path).unwrap();
    let config = GovernanceConfig::parse(&yaml).unwrap();
    GovernanceEngine::from_config(&config).unwrap()
}

#[test]
fn broad_catalog_detects_both_aws_credential_components() {
    let engine = shipped_engine();
    let scanner = engine.secret_scanner().unwrap();
    let access_id = GovernedInput::prompt_text("AKIAIOSFODNN7EXAMPLE".to_owned());
    assert_eq!(scanner.detect(&access_id).unwrap().pattern.id, "aws-access-key");

    let secret = GovernedInput::tool_arguments(McpToolInput::new(serde_json::json!({
        "aws_secret_access_key": "Ab9/".repeat(10)
    })));
    assert_eq!(scanner.detect(&secret).unwrap().pattern.id, "aws-secret-key");
}

#[test]
fn broad_catalog_detects_other_documented_provider_credentials() {
    let engine = shipped_engine();
    let scanner = engine.secret_scanner().unwrap();
    for (value, expected) in [
        (
            "ghp_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789",
            "github-token-classic",
        ),
        ("sk-ant-api03-AbCdEfGhIjKlMnOpQrStUv", "anthropic-api-key"),
    ] {
        let input = GovernedInput::prompt_text(value.to_owned());
        assert_eq!(scanner.detect(&input).unwrap().pattern.id, expected);
    }
}
