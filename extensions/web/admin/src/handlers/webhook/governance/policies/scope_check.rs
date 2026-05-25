//! `scope_check`: gate admin-only tools by agent scope.
//!
//! Reads the OAuth scope label ("admin" / "user" / "unknown") from
//! `ctx.extras.scope_label` — the template plumbs this in
//! [`super::super::handler`] before invoking the chain. Configurable via:
//!
//! ```yaml
//! - id: scope_check
//!   admin_only_prefixes:
//!     - "mcp__systemprompt__"
//! ```

use std::borrow::Cow;

use serde_yaml::Value as YamlValue;
use systemprompt::identifiers::{McpToolName, PolicyId};
use systemprompt_security::authz::{Decision, DenyReason, MatchedBy};
use systemprompt_security::policy::{GovernancePolicy, PolicyContext};

use super::super::policy::PolicyRegistration;
use crate::types::{SCOPE_ADMIN, SCOPE_UNKNOWN};

const ID: &str = "scope_check";
const DEFAULT_ADMIN_ONLY_PREFIXES: &[&str] = &["mcp__systemprompt__"];

#[derive(Debug)]
pub struct ScopeCheck {
    admin_only_prefixes: Vec<String>,
}

impl ScopeCheck {
    fn from_yaml(v: &YamlValue) -> Self {
        let prefixes = v
            .get("admin_only_prefixes")
            .and_then(|s| s.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|p| p.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
            .unwrap_or_else(|| {
                DEFAULT_ADMIN_ONLY_PREFIXES
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect()
            });
        Self {
            admin_only_prefixes: prefixes,
        }
    }
}

/// The template handler injects the OAuth scope label
/// (`SCOPE_LABEL_KEY`) into the wrapped `tool_input` value before invoking
/// the chain. Core's [`PolicyContext`] doesn't carry deployment-specific
/// principal metadata, so the JSON-boundary wrapper is the agreed plumbing.
fn scope_label<'a>(ctx: &'a PolicyContext<'_>) -> &'a str {
    ctx.tool_input
        .as_str(super::super::SCOPE_LABEL_KEY)
        .unwrap_or(SCOPE_UNKNOWN)
}

impl GovernancePolicy for ScopeCheck {
    fn id(&self) -> PolicyId {
        PolicyId::new(ID)
    }
    fn name(&self) -> &'static str {
        "Scope Check"
    }
    fn description(&self) -> &'static str {
        "Block non-admin agents from calling tools whose name starts with an \
         admin-only prefix (default: mcp__systemprompt__)."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> Decision {
        let scope = scope_label(ctx);
        if scope == SCOPE_ADMIN {
            return Decision::Allow {
                matched_by: MatchedBy::PolicyAllow {
                    policy_id: PolicyId::new(ID),
                    detail: Cow::Borrowed("admin scope grants unrestricted tool access"),
                },
            };
        }

        let tool_str = ctx.tool.as_str();
        let requires_admin = self
            .admin_only_prefixes
            .iter()
            .any(|prefix| tool_str.starts_with(prefix.as_str()));

        if requires_admin {
            return Decision::Deny {
                reason: DenyReason::ScopeViolation {
                    tool: McpToolName::new(tool_str),
                    missing_scope: SCOPE_ADMIN.to_string(),
                },
            };
        }

        let detail = if scope == SCOPE_UNKNOWN {
            Cow::Borrowed("Agent scope could not be resolved; allowed for non-admin tool")
        } else {
            Cow::Owned(format!("{scope} scope is allowed for tool: {tool_str}"))
        };
        Decision::Allow {
            matched_by: MatchedBy::PolicyAllow {
                policy_id: PolicyId::new(ID),
                detail,
            },
        }
    }
}

inventory::submit! {
    PolicyRegistration {
        id: ID,
        factory: |v| Box::new(ScopeCheck::from_yaml(v)),
        source_path: file!(),
    }
}
