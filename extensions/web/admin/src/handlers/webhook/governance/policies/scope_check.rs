//! `scope_check`: gate admin-only tools by agent scope.
//!
//! Configurable via:
//! ```yaml
//! - id: scope_check
//!   admin_only_prefixes:
//!     - "mcp__systemprompt__"
//!     - "mcp__skill-manager__"
//! ```

use std::borrow::Cow;

use serde_yaml::Value as YamlValue;

use super::super::policy::{Policy, PolicyContext, PolicyOutcome, PolicyRegistration};
use crate::types::{SCOPE_ADMIN, SCOPE_UNKNOWN};

const ID: &str = "scope_check";
const DEFAULT_ADMIN_ONLY_PREFIXES: &[&str] = &["mcp__systemprompt__", "mcp__skill-manager__"];

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

impl Policy for ScopeCheck {
    fn id(&self) -> &'static str {
        ID
    }
    fn name(&self) -> &'static str {
        "Scope Check"
    }
    fn description(&self) -> &'static str {
        "Block non-admin agents from calling tools whose name starts with an \
         admin-only prefix (default: mcp__systemprompt__, mcp__skill-manager__)."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyOutcome {
        if ctx.agent_scope == SCOPE_ADMIN {
            return PolicyOutcome::Allow {
                detail: Cow::Borrowed("admin scope grants unrestricted tool access"),
            };
        }

        let requires_admin = self
            .admin_only_prefixes
            .iter()
            .any(|prefix| ctx.tool_name.starts_with(prefix.as_str()));

        if requires_admin {
            return PolicyOutcome::Deny {
                reason: format!(
                    "{} scope cannot access admin-only tool: {}",
                    ctx.agent_scope, ctx.tool_name
                ),
                detail: Cow::Owned(format!(
                    "{} scope cannot access tools matching admin-only prefixes",
                    ctx.agent_scope
                )),
            };
        }

        if ctx.agent_scope == SCOPE_UNKNOWN {
            PolicyOutcome::Allow {
                detail: Cow::Borrowed(
                    "Agent scope could not be resolved; allowed for non-admin tool",
                ),
            }
        } else {
            PolicyOutcome::Allow {
                detail: Cow::Owned(format!(
                    "{} scope is allowed for tool: {}",
                    ctx.agent_scope, ctx.tool_name
                )),
            }
        }
    }
}

inventory::submit! {
    PolicyRegistration {
        id: ID,
        factory: |v| Box::new(ScopeCheck::from_yaml(v)),
    }
}
