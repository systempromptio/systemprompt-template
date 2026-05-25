//! `secret_scan`: refuse tool calls whose `tool_input` contains a plaintext
//! credential matching one of the built-in patterns. The pattern list ships
//! with the binary; per-deployment additions go in
//! `services/governance/config.yaml -> policies[id=secret_scan].extra_patterns`.

use std::borrow::Cow;

use serde_yaml::Value as YamlValue;
use systemprompt::identifiers::{PolicyId, SecretPatternId};
use systemprompt_security::authz::{Decision, DenyReason, MatchedBy};
use systemprompt_security::policy::{GovernancePolicy, PolicyContext, SecretLocation};

use super::super::policy::PolicyRegistration;
use super::super::secrets::detect_secrets;

const ID: &str = "secret_scan";

#[derive(Debug)]
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
        if let Some((name, redacted)) = detect_secrets(Some(tool_input_value)) {
            return Decision::Deny {
                reason: DenyReason::SecretLeak {
                    pattern_id: SecretPatternId::new(name.clone()),
                    location: SecretLocation::new("tool_input", redacted),
                },
            };
        }
        let mut strings = Vec::new();
        collect_strings(tool_input_value, &mut strings);
        for s in &strings {
            for (name, prefix) in &self.extra_patterns {
                if s.contains(prefix.as_str()) {
                    return Decision::Deny {
                        reason: DenyReason::SecretLeak {
                            pattern_id: SecretPatternId::new(name.clone()),
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
        source_path: file!(),
    }
}
