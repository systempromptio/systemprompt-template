//! Gateway [`SafetyScanner`] implementation for the systemprompt template.
//!
//! [`SecretsScanner`] flags plaintext credentials (GitHub / Anthropic / AWS /
//! Stripe / … tokens, private keys, DB URLs with passwords) carried in
//! inference content, reusing the same `SECRET_PATTERNS` that the governance
//! tool-use webhook applies. It registers through `register_safety_scanner!`
//! under the name `secrets`; the gateway runs it for any policy whose
//! `safety.scanners` lists it and blocks the request when `safety
//! .block_categories` includes `secret`.

use systemprompt::ai::{Finding, SafetyScanner, Severity, register_safety_scanner};
use systemprompt::models::wire::canonical::{
    CanonicalContent, CanonicalRequest, CanonicalResponse,
};

use crate::handlers::webhook::governance::secrets::scan_str_for_secret;

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

    async fn scan_request(&self, req: &CanonicalRequest) -> Vec<Finding> {
        scan("request", &req.flatten_text())
    }

    async fn scan_response_final(&self, response: &CanonicalResponse) -> Vec<Finding> {
        let mut text = String::new();
        for part in &response.content {
            if let CanonicalContent::Text(t) = part {
                text.push_str(t);
                text.push('\n');
            }
        }
        scan("response", &text)
    }
}

fn scan(phase: &'static str, text: &str) -> Vec<Finding> {
    scan_str_for_secret(text).map_or_else(Vec::new, |excerpt| {
        vec![Finding {
            phase,
            severity: Severity::High,
            category: "secret".to_owned(),
            excerpt: Some(excerpt),
            scanner: "secrets",
        }]
    })
}

register_safety_scanner!(SecretsScanner::new, name = "secrets");
