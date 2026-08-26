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

/// PII scanner for the categories core's heuristic does not carry: formatted
/// US SSNs and international-format phone numbers.
///
/// Both directions, unlike [`SecretsScanner`]: the governance `secret_scan`
/// stage owns request-side *credentials*, but no ingress plane scans for PII,
/// so there is no second-verdict overlap to avoid. Detection is deliberately
/// conservative — only the unambiguous shapes — because a false positive here
/// blocks a customer's request, and "looks a bit like a phone number" is most
/// ten-digit figures in an analytics conversation.
#[derive(Debug, Clone, Copy, Default)]
pub struct PiiScanner;

impl PiiScanner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SafetyScanner for PiiScanner {
    fn name(&self) -> &'static str {
        "pii_extended"
    }

    async fn scan_request(&self, req: &CanonicalRequest) -> Vec<Finding> {
        req.message_units()
            .into_iter()
            .flat_map(|unit| pii_findings(&unit, "request"))
            .collect()
    }

    async fn scan_response_final(&self, response: &CanonicalResponse) -> Vec<Finding> {
        response
            .content_units()
            .into_iter()
            .flat_map(|unit| pii_findings(&unit, "response"))
            .collect()
    }
}

fn pii_findings(text: &str, phase: &'static str) -> Vec<Finding> {
    let mut findings = Vec::new();
    if let Some(excerpt) = find_ssn(text) {
        findings.push(Finding {
            phase,
            severity: Severity::High,
            category: "pii_ssn".to_owned(),
            excerpt: Some(excerpt),
            scanner: "pii_extended",
        });
    }
    if let Some(excerpt) = find_phone(text) {
        findings.push(Finding {
            phase,
            severity: Severity::Medium,
            category: "pii_phone".to_owned(),
            excerpt: Some(excerpt),
            scanner: "pii_extended",
        });
    }
    findings
}

// Why: only the canonical AAA-GG-SSSS shape with digit-boundary checks, and
// the SSA's never-issued ranges excluded — 000/666/9xx areas, 00 group, 0000
// serial. An excerpt masks all but the last four, the convention SSNs are
// displayed with everywhere else.
fn find_ssn(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(10) {
        let w = &bytes[i..i + 11];
        let shape_ok = w[..3].iter().all(u8::is_ascii_digit)
            && w[3] == b'-'
            && w[4..6].iter().all(u8::is_ascii_digit)
            && w[6] == b'-'
            && w[7..].iter().all(u8::is_ascii_digit);
        if !shape_ok || digit_adjacent(bytes, i, i + 11) {
            continue;
        }
        let area = &text[i..i + 3];
        let group = &text[i + 4..i + 6];
        let serial = &text[i + 7..i + 11];
        if area == "000" || area == "666" || area.starts_with('9') {
            continue;
        }
        if group == "00" || serial == "0000" {
            continue;
        }
        return Some(format!("***-**-{serial}"));
    }
    None
}

// Why: E.164-style only — a `+` followed by 8..15 digits with optional
// space/dash/paren separators. Bare ten-digit runs are not matched: order
// ids, timestamps, and token counts all look like them.
fn find_phone(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    for (i, _) in bytes.iter().enumerate().filter(|&(_, &b)| b == b'+') {
        let mut digits = 0usize;
        let mut end = i + 1;
        for (j, &b) in bytes.iter().enumerate().skip(i + 1) {
            match b {
                b'0'..=b'9' => {
                    digits += 1;
                    end = j + 1;
                },
                b' ' | b'-' | b'(' | b')' | b'.' => {},
                _ => break,
            }
            if digits > 15 {
                break;
            }
        }
        if (8..=15).contains(&digits) && !digit_adjacent(bytes, i, end) {
            let tail: String = text[i..end]
                .chars()
                .filter(char::is_ascii_digit)
                .skip(digits.saturating_sub(3))
                .collect();
            return Some(format!("+…{tail}"));
        }
    }
    None
}

// Why: a match butted against more digits is part of a longer number — an id,
// a hash, a timestamp — not the entity being looked for.
fn digit_adjacent(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).map(|i| bytes[i]);
    let after = bytes.get(end).copied();
    before.is_some_and(|b| b.is_ascii_digit()) || after.is_some_and(|b| b.is_ascii_digit())
}

register_safety_scanner!(PiiScanner::new, name = "pii_extended");
