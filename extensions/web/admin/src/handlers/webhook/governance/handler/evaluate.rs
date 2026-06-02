//! Policy-chain evaluation for the governance webhook.
//!
//! Iterates the configured policy chain in order and records a per-entry trace
//! so the audit row preserves the full evaluation order, not just the first
//! deny. The handler owns request flow and auditing; this module owns the
//! decision logic.

use systemprompt::identifiers::{McpToolName, SessionId, UserId};
use systemprompt_security::authz::{Decision, MatchedBy};
use systemprompt_security::policy::{types::AccessScope, AgentScope, McpToolInput, PolicyContext};

use super::super::policy::{self, PolicyConfig};
use super::super::types::{ChainEntryOutcome, ChainEntryResult};

pub(super) struct EvaluateInput<'a> {
    pub(super) tool_name: &'a str,
    pub(super) session_id: &'a SessionId,
    pub(super) user_id: &'a UserId,
    pub(super) access_scope: AccessScope,
    pub(super) tool_input: Option<&'a serde_json::Value>,
}

pub(super) fn evaluate(input: &EvaluateInput<'_>) -> (Decision, Vec<ChainEntryOutcome>) {
    let tool_input = McpToolInput::new(
        input
            .tool_input
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())),
    );

    let ctx = PolicyContext {
        tool: McpToolName::new(input.tool_name),
        agent_scope: AgentScope::User {
            user_id: input.user_id.clone(),
        },
        access_scope: input.access_scope,
        session_id: input.session_id,
        user_id: input.user_id,
        tool_input: &tool_input,
    };

    let mut chain_trace: Vec<ChainEntryOutcome> = Vec::new();
    let mut denied: Option<Decision> = None;

    let chain = policy::chain();
    for (cfg, policy) in chain.iter() {
        if !cfg.enabled {
            chain_trace.push(disabled_entry(cfg));
            continue;
        }
        if denied.is_some() {
            chain_trace.push(skipped_after_deny(cfg));
            continue;
        }
        let decision = policy.evaluate(&ctx);
        match &decision {
            Decision::Allow { matched_by } => {
                chain_trace.push(ChainEntryOutcome {
                    policy_id: policy.id(),
                    result: ChainEntryResult::Pass,
                    detail: allow_detail(matched_by),
                });
            }
            Decision::Deny { reason } => {
                chain_trace.push(ChainEntryOutcome {
                    policy_id: policy.id(),
                    result: ChainEntryResult::Fail,
                    detail: reason.to_string(),
                });
                denied = Some(decision);
            }
        }
    }
    drop(chain);

    let final_decision = denied.unwrap_or(Decision::Allow {
        matched_by: MatchedBy::DefaultIncluded,
    });
    (final_decision, chain_trace)
}

fn disabled_entry(cfg: &PolicyConfig) -> ChainEntryOutcome {
    ChainEntryOutcome {
        policy_id: systemprompt::identifiers::PolicyId::new(cfg.id.clone()),
        result: ChainEntryResult::Skip,
        detail: "Policy disabled in services/governance/config.yaml".to_string(),
    }
}

fn skipped_after_deny(cfg: &PolicyConfig) -> ChainEntryOutcome {
    ChainEntryOutcome {
        policy_id: systemprompt::identifiers::PolicyId::new(cfg.id.clone()),
        result: ChainEntryResult::Skip,
        detail: "Skipped — already denied by an earlier policy".to_string(),
    }
}

fn allow_detail(matched_by: &MatchedBy) -> String {
    match matched_by {
        MatchedBy::PolicyAllow { detail, .. } => detail.to_string(),
        MatchedBy::UserAllow => "user allow".to_string(),
        MatchedBy::RoleAllow { role } => format!("role allow: {role}"),
        MatchedBy::DefaultIncluded => "default included".to_string(),
    }
}
