//! Pipeline driver. Iterates the configured policy chain in order, calling
//! [`Policy::evaluate`] for each enabled policy and stopping on the first
//! deny. Every policy's outcome lands in `evaluated_rules` so the audit log
//! preserves the full evaluation trace, not just the first-deny.

use std::borrow::Cow;

use super::policy::{self, PolicyContext, PolicyOutcome};
use super::types::{EvaluatedRule, GovernanceContext, GovernanceDecision, RuleEvaluation};

pub(super) fn evaluate(ctx: &GovernanceContext<'_>) -> RuleEvaluation {
    let policy_ctx = PolicyContext {
        tool_name: ctx.tool_name,
        agent_scope: ctx.agent_scope,
        session_id: ctx.session_id,
        user_id: ctx.user_id,
        tool_input: ctx.tool_input,
    };

    let mut rules: Vec<EvaluatedRule> = Vec::new();
    let mut denied: Option<(Cow<'static, str>, Cow<'static, str>)> = None;

    let chain = policy::chain();
    for (cfg, policy) in chain.iter() {
        if !cfg.enabled {
            rules.push(EvaluatedRule {
                rule: leak_id(policy.id()),
                result: "skip",
                detail: Cow::Borrowed("Policy disabled in services/governance/config.yaml"),
            });
            continue;
        }

        if denied.is_some() {
            rules.push(EvaluatedRule {
                rule: leak_id(policy.id()),
                result: "skip",
                detail: Cow::Borrowed("Skipped — already denied"),
            });
            continue;
        }

        match policy.evaluate(&policy_ctx) {
            PolicyOutcome::Allow { detail } => {
                rules.push(EvaluatedRule {
                    rule: leak_id(policy.id()),
                    result: "pass",
                    detail,
                });
            }
            PolicyOutcome::Deny { reason, detail } => {
                rules.push(EvaluatedRule {
                    rule: leak_id(policy.id()),
                    result: "fail",
                    detail,
                });
                denied = Some((Cow::Owned(reason), Cow::Owned(policy.id().to_string())));
            }
        }
    }
    drop(chain);

    if let Some((reason, policy)) = denied {
        RuleEvaluation {
            decision: GovernanceDecision::Deny,
            reason,
            policy,
            rules,
        }
    } else {
        RuleEvaluation {
            decision: GovernanceDecision::Allow,
            reason: Cow::Borrowed("All governance rules passed"),
            policy: Cow::Borrowed("default_allow"),
            rules,
        }
    }
}

/// `Policy::id()` returns a `&'static str`. `EvaluatedRule.rule` is also
/// `&'static str`, so we just thread the same lifetime through. This helper
/// is here to make that intent obvious at the call sites.
const fn leak_id(id: &'static str) -> &'static str {
    id
}
