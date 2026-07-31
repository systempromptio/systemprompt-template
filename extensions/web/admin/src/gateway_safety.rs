//! Gateway [`SafetyScanner`] implementation for the systemprompt template.
//!
//! [`SecretsScanner`] flags plaintext credentials (GitHub / Anthropic / AWS /
//! Stripe / … tokens, private keys, DB URLs with passwords) leaving the
//! gateway in a model reply, reusing the same `SECRET_PATTERNS` that the
//! governance chain applies on the way in. It registers through
//! `register_safety_scanner!` under the name `secrets`; the gateway runs it
//! for any policy whose `safety.scanners` lists it and blocks the reply when
//! `safety.block_response_categories` includes `secret`.
//!
//! **Egress only.** The `secret_scan` governance policy already scans request
//! content, runs first (`enforce_governance` precedes
//! `enforce_request_safety`), and is first-deny-wins, so scanning the request
//! here too produced a second verdict on identical bytes under different
//! config — two owners for one question, and a request that could be denied by
//! whichever plane the operator had not thought to configure. Responses have
//! no such overlap: the governance chain is request-only, so this is the sole
//! thing standing between a model that echoes a credential and the client.

use systemprompt::ai::{Finding, SafetyScanner, Severity, register_safety_scanner};
use systemprompt::models::wire::canonical::{CanonicalRequest, CanonicalResponse};

use systemprompt_security::policy::secrets::scan_str_for_secret;

#[derive(Debug, Clone, Copy, Default)]
pub struct SecretsScanner;

impl SecretsScanner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SafetyScanner for SecretsScanner {
    fn name(&self) -> &'static str {
        "secrets"
    }

    async fn scan_request(&self, _req: &CanonicalRequest) -> Vec<Finding> {
        Vec::new()
    }

    async fn scan_response_final(&self, response: &CanonicalResponse) -> Vec<Finding> {
        let mut findings = Vec::new();
        for unit in response.content_units() {
            findings.extend(scan(&unit));
        }
        findings
    }
}

fn scan(text: &str) -> Vec<Finding> {
    scan_str_for_secret(text).map_or_else(Vec::new, |excerpt| {
        vec![Finding {
            phase: "response",
            severity: Severity::High,
            category: "secret".to_owned(),
            excerpt: Some(excerpt),
            scanner: "secrets",
        }]
    })
}

register_safety_scanner!(SecretsScanner::new, name = "secrets");
