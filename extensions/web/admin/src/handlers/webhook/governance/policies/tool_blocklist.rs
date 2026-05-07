//! `tool_blocklist`: block destructive tool names for non-admin agents.
//!
//! Configurable via:
//! ```yaml
//! - id: tool_blocklist
//!   patterns: ["delete", "drop", "destroy"]
//! ```

use std::borrow::Cow;

use serde_yaml::Value as YamlValue;

use super::super::policy::{Policy, PolicyContext, PolicyOutcome, PolicyRegistration};
use crate::types::SCOPE_ADMIN;

const ID: &str = "tool_blocklist";
const DEFAULT_PATTERNS: &[&str] = &["delete", "drop", "destroy"];

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

impl Policy for ToolBlocklist {
    fn id(&self) -> &'static str {
        ID
    }
    fn name(&self) -> &'static str {
        "Tool Blocklist"
    }
    fn description(&self) -> &'static str {
        "Block tool names containing destructive substrings (e.g. delete/drop/destroy) \
         for any agent without admin scope."
    }
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyOutcome {
        let matched = self
            .patterns
            .iter()
            .find(|p| ctx.tool_name.contains(p.as_str()));

        match matched {
            Some(p) if ctx.agent_scope != SCOPE_ADMIN => PolicyOutcome::Deny {
                reason: format!(
                    "Destructive tool '{}' blocked for {} scope (matched pattern '{}')",
                    ctx.tool_name, ctx.agent_scope, p
                ),
                detail: Cow::Owned(format!(
                    "Tool '{}' matches blocklist pattern '{}'",
                    ctx.tool_name, p
                )),
            },
            _ => PolicyOutcome::Allow {
                detail: Cow::Borrowed("Tool not on restricted list"),
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
