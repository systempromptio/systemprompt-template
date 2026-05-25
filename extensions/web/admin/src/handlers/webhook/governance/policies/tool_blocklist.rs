//! `tool_blocklist`: block destructive tool names for non-admin agents.
//!
//! Configurable via:
//! ```yaml
//! - id: tool_blocklist
//!   patterns: ["delete", "drop", "destroy"]
//! ```

use std::borrow::Cow;

use serde_yaml::Value as YamlValue;
use systemprompt::identifiers::{McpToolName, PolicyId};
use systemprompt_security::authz::{Decision, DenyReason, MatchedBy};
use systemprompt_security::policy::{GovernancePolicy, PolicyContext};

use super::super::policy::PolicyRegistration;
use crate::types::SCOPE_ADMIN;

const ID: &str = "tool_blocklist";
const DEFAULT_PATTERNS: &[&str] = &["delete", "drop", "destroy"];

#[derive(Debug)]
pub struct ToolBlocklist {
    patterns: Vec<String>,
}

impl ToolBlocklist {
    fn from_yaml(v: &YamlValue) -> Self {
        let patterns = v
            .get("patterns")
            .and_then(|s| s.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|p| p.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .filter(|v: &Vec<String>| !v.is_empty())
            .unwrap_or_else(|| DEFAULT_PATTERNS.iter().map(|s| (*s).to_string()).collect());
        Self { patterns }
    }
}

/// See [`super::scope_check::scope_label`] for the rationale on reading the
/// scope label from the wrapped tool input.
fn scope_label<'a>(ctx: &'a PolicyContext<'_>) -> &'a str {
    ctx.tool_input
        .as_str(super::super::SCOPE_LABEL_KEY)
        .unwrap_or("unknown")
}

impl GovernancePolicy for ToolBlocklist {
    fn id(&self) -> PolicyId {
        PolicyId::new(ID)
    }
    fn name(&self) -> &'static str {
        "Tool Blocklist"
    }
    fn description(&self) -> &'static str {
        "Block tool names containing destructive substrings (e.g. delete/drop/destroy) \
         for any agent without admin scope."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> Decision {
        let tool_str = ctx.tool.as_str();
        let scope = scope_label(ctx);
        let matched = self
            .patterns
            .iter()
            .find(|p| tool_str.contains(p.as_str()));

        match matched {
            Some(p) if scope != SCOPE_ADMIN => Decision::Deny {
                reason: DenyReason::ToolBlocked {
                    tool: McpToolName::new(tool_str),
                    list_id: p.clone(),
                },
            },
            _ => Decision::Allow {
                matched_by: MatchedBy::PolicyAllow {
                    policy_id: PolicyId::new(ID),
                    detail: Cow::Borrowed("Tool not on restricted list"),
                },
            },
        }
    }
}

inventory::submit! {
    PolicyRegistration {
        id: ID,
        factory: |v| Box::new(ToolBlocklist::from_yaml(v)),
        source_path: file!(),
    }
}
