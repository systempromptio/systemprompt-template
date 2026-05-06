//! `secret_scan`: refuse tool calls whose `tool_input` contains a plaintext
//! credential matching one of the built-in patterns. The pattern list ships
//! with the binary; per-deployment additions go in
//! `services/governance/config.yaml -> policies[id=secret_scan].extra_patterns`.

use std::borrow::Cow;

use serde_yaml::Value as YamlValue;

use super::super::policy::{Policy, PolicyContext, PolicyOutcome, PolicyRegistration};
use super::super::secrets::detect_secrets;

const ID: &str = "secret_scan";

pub struct SecretScan {
    extra_patterns: Vec<(String, String)>,
}

impl SecretScan {
    fn from_yaml(v: &YamlValue) -> Self {
        let extra = v
            .get("extra_patterns")
            .and_then(|s| s.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|p| {
                        let name = p.get("name").and_then(|n| n.as_str())?;
                        let prefix = p.get("prefix").and_then(|n| n.as_str())?;
                        Some((name.to_string(), prefix.to_string()))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Self {
            extra_patterns: extra,
        }
    }
}

impl Policy for SecretScan {
    fn id(&self) -> &'static str {
        ID
    }
    fn name(&self) -> &'static str {
        "Secret Scan"
    }
    fn description(&self) -> &'static str {
        "Block tool calls whose input contains an AWS key, GitHub PAT, PEM block, \
         connection string, or other plaintext credential pattern."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyOutcome {
        if let Some((name, redacted)) = detect_secrets(ctx.tool_input) {
            return PolicyOutcome::Deny {
                reason: format!(
                    "SECURITY BREACH: Plaintext secret detected in tool input — {name} ({redacted})"
                ),
                detail: Cow::Owned(format!(
                    "Plaintext secret detected: {name} — matched '{redacted}' in tool_input"
                )),
            };
        }
        // Extra (config-supplied) patterns: same prefix-match contract.
        if let Some(input) = ctx.tool_input {
            let mut strings = Vec::new();
            collect_strings(input, &mut strings);
            for s in &strings {
                for (name, prefix) in &self.extra_patterns {
                    if s.contains(prefix.as_str()) {
                        return PolicyOutcome::Deny {
                            reason: format!(
                                "SECURITY BREACH: Plaintext secret detected in tool input — {name}"
                            ),
                            detail: Cow::Owned(format!("Custom pattern matched: {name}")),
                        };
                    }
                }
            }
        }
        PolicyOutcome::Allow {
            detail: Cow::Borrowed("No plaintext secrets detected in tool input"),
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
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_strings(v, out);
            }
        }
        _ => {}
    }
}

inventory::submit! {
    PolicyRegistration {
        id: ID,
        factory: |v| Box::new(SecretScan::from_yaml(v)),
    }
}
