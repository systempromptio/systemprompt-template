//! `secret_scan`: refuse tool calls whose `tool_input` contains a plaintext
//! credential matching one of the built-in patterns. The pattern list ships
//! with the binary; per-deployment additions go in
//! `services/governance/config.yaml ->
//! policies[id=secret_scan].extra_patterns`.

use std::borrow::Cow;

use serde_yaml::Value as YamlValue;
use systemprompt::identifiers::{PolicyId, SecretPatternId};
use systemprompt_security::authz::{Decision, DenyReason, MatchedBy};
use systemprompt_security::policy::{GovernancePolicy, PolicyContext, SecretLocation};

use super::super::policy::PolicyRegistration;
use super::super::secrets::detect_secrets;

const ID: &str = "secret_scan";

/// Operator-defined extra pattern loaded from
/// `services/governance/config.yaml`. `id` is derived from `name` at load
/// time via [`slugify`] so the runtime referent stays stable; collisions are
/// logged and the duplicate is dropped.
#[derive(Debug, Clone)]
struct ExtraPattern {
    id: String,
    name: String,
    prefix: String,
}

#[derive(Debug)]
pub(super) struct SecretScan {
    extra_patterns: Vec<ExtraPattern>,
}

impl SecretScan {
    fn from_yaml(v: &YamlValue) -> Self {
        let extras = v
            .get("extra_patterns")
            .and_then(|s| s.as_sequence())
            .map(|seq| {
                let mut out: Vec<ExtraPattern> = Vec::new();
                for entry in seq {
                    let Some(name) = entry.get("name").and_then(|n| n.as_str()) else {
                        continue;
                    };
                    let Some(prefix) = entry.get("prefix").and_then(|n| n.as_str()) else {
                        continue;
                    };
                    let id = slugify(name);
                    if out.iter().any(|p| p.id == id) {
                        tracing::error!(
                            extra_pattern_name = %name,
                            extra_pattern_id = %id,
                            "secret_scan: duplicate extra_pattern id derived from name; \
                             keeping first occurrence and skipping the duplicate"
                        );
                        continue;
                    }
                    out.push(ExtraPattern {
                        id,
                        name: name.to_owned(),
                        prefix: prefix.to_owned(),
                    });
                }
                out
            })
            .unwrap_or_default();
        Self {
            extra_patterns: extras,
        }
    }
}

impl GovernancePolicy for SecretScan {
    fn id(&self) -> PolicyId {
        PolicyId::new(ID)
    }
    fn name(&self) -> &'static str {
        "Secret Scan"
    }
    fn description(&self) -> &'static str {
        "Block tool calls whose input contains an AWS key, GitHub PAT, PEM block, \
         connection string, or other plaintext credential pattern."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> Decision {
        let tool_input_value = ctx.tool_input.as_value();
        if let Some((pattern, redacted)) = detect_secrets(Some(tool_input_value)) {
            return Decision::Deny {
                reason: DenyReason::SecretLeak {
                    pattern_id: SecretPatternId::new(pattern.id),
                    pattern_name: Cow::Borrowed(pattern.name),
                    location: SecretLocation::new("tool_input", redacted),
                },
            };
        }
        let mut strings = Vec::new();
        collect_strings(tool_input_value, &mut strings);
        for s in &strings {
            for extra in &self.extra_patterns {
                if s.contains(extra.prefix.as_str()) {
                    return Decision::Deny {
                        reason: DenyReason::SecretLeak {
                            pattern_id: SecretPatternId::new(extra.id.clone()),
                            pattern_name: Cow::Owned(extra.name.clone()),
                            location: SecretLocation::new("tool_input", "custom_pattern"),
                        },
                    };
                }
            }
        }
        Decision::Allow {
            matched_by: MatchedBy::PolicyAllow {
                policy_id: PolicyId::new(ID),
                detail: Cow::Borrowed("No plaintext secrets detected in tool input"),
            },
        }
    }
}

fn collect_strings(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_strings(v, out);
            }
        },
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_strings(v, out);
            }
        },
        _ => {},
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_dash = false;
    for ch in input.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else if ch.is_whitespace() || matches!(ch, '_' | '-' | '/' | '(' | ')' | '.') {
            Some('-')
        } else {
            None
        };
        if let Some(c) = mapped {
            if c == '-' {
                if !last_was_dash && !out.is_empty() {
                    out.push('-');
                    last_was_dash = true;
                }
            } else {
                out.push(c);
                last_was_dash = false;
            }
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

inventory::submit! {
    PolicyRegistration {
        id: ID,
        factory: |v| Box::new(SecretScan::from_yaml(v)),
        source_path: file!(),
    }
}
