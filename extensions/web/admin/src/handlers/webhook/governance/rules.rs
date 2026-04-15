use std::borrow::Cow;

use super::rate_limit;
use super::secrets::detect_secrets;
use super::types::{EvaluatedRule, GovernanceContext, GovernanceDecision, RuleEvaluation};
use crate::types::{SCOPE_ADMIN, SCOPE_UNKNOWN};

const ADMIN_ONLY_TOOL_PREFIXES: &[&str] = &["mcp__systemprompt__", "mcp__skill-manager__"];

pub(super) fn evaluate(ctx: &GovernanceContext<'_>) -> RuleEvaluation {
    let mut rules = Vec::new();
    let mut denied = false;
    let mut deny_reason = Cow::Borrowed("");
    let mut deny_policy = Cow::Borrowed("");

    evaluate_secret_detection(
        ctx,
        &mut rules,
        &mut denied,
        &mut deny_reason,
        &mut deny_policy,
    );
    evaluate_scope(
        ctx,
        &mut rules,
        &mut denied,
        &mut deny_reason,
        &mut deny_policy,
    );
    evaluate_blocklist(
        ctx,
        &mut rules,
        &mut denied,
        &mut deny_reason,
        &mut deny_policy,
    );
    evaluate_rate_limit(
        ctx,
        &mut rules,
        &mut denied,
        &mut deny_reason,
        &mut deny_policy,
    );

    if denied {
        RuleEvaluation {
            decision: GovernanceDecision::Deny,
            reason: deny_reason,
            policy: deny_policy,
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

fn evaluate_secret_detection(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut Cow<'static, str>,
    deny_policy: &mut Cow<'static, str>,
) {
    match detect_secrets(ctx.tool_input) {
        Some((pattern_name, redacted)) => {
            *denied = true;
            *deny_reason = Cow::Owned(format!(
                "SECURITY BREACH: Plaintext secret detected in tool input \
                 — {pattern_name} ({redacted})"
            ));
            *deny_policy = Cow::Borrowed("secret_injection");
            rules.push(EvaluatedRule {
                rule: "secret_detection",
                result: "fail",
                detail: Cow::Owned(format!(
                    "Plaintext secret detected: {pattern_name} — matched '{redacted}' in tool_input"
                )),
            });
        }
        None => {
            rules.push(EvaluatedRule {
                rule: "secret_detection",
                result: "pass",
                detail: Cow::Borrowed("No plaintext secrets detected in tool input"),
            });
        }
    }
}

fn evaluate_scope(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut Cow<'static, str>,
    deny_policy: &mut Cow<'static, str>,
) {
    if *denied {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "skip",
            detail: Cow::Borrowed("Skipped — already denied"),
        });
        return;
    }

    if ctx.agent_scope == SCOPE_ADMIN {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "pass",
            detail: Cow::Borrowed("admin scope grants unrestricted tool access"),
        });
        return;
    }

    if ctx.agent_scope == SCOPE_UNKNOWN {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "warn",
            detail: Cow::Borrowed("Agent scope could not be resolved, treating as restricted"),
        });
        return;
    }

    let requires_admin = ADMIN_ONLY_TOOL_PREFIXES
        .iter()
        .any(|prefix| ctx.tool_name.starts_with(prefix));

    if requires_admin {
        *denied = true;
        *deny_reason = Cow::Owned(format!(
            "{} scope cannot access admin-only tool: {}",
            ctx.agent_scope, ctx.tool_name
        ));
        *deny_policy = Cow::Borrowed("scope_restriction");
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "fail",
            detail: Cow::Owned(format!(
                "{} scope cannot access tools matching admin-only prefixes",
                ctx.agent_scope
            )),
        });
    } else {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "pass",
            detail: Cow::Owned(format!(
                "{} scope is allowed for tool: {}",
                ctx.agent_scope, ctx.tool_name
            )),
        });
    }
}

fn evaluate_blocklist(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut Cow<'static, str>,
    deny_policy: &mut Cow<'static, str>,
) {
    if *denied {
        rules.push(EvaluatedRule {
            rule: "tool_blocklist",
            result: "skip",
            detail: Cow::Borrowed("Skipped — already denied"),
        });
        return;
    }

    let is_destructive = ctx.tool_name.contains("delete")
        || ctx.tool_name.contains("drop")
        || ctx.tool_name.contains("destroy");

    if is_destructive && ctx.agent_scope != SCOPE_ADMIN {
        *denied = true;
        *deny_reason = Cow::Owned(format!(
            "Destructive tool '{}' blocked for {} scope",
            ctx.tool_name, ctx.agent_scope
        ));
        *deny_policy = Cow::Borrowed("tool_blocklist");
        rules.push(EvaluatedRule {
            rule: "tool_blocklist",
            result: "fail",
            detail: Cow::Owned(format!(
                "Tool '{}' matches destructive pattern blocklist",
                ctx.tool_name
            )),
        });
    } else {
        rules.push(EvaluatedRule {
            rule: "tool_blocklist",
            result: "pass",
            detail: Cow::Borrowed("Tool not on restricted list"),
        });
    }
}

fn evaluate_rate_limit(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut Cow<'static, str>,
    deny_policy: &mut Cow<'static, str>,
) {
    if *denied {
        rules.push(EvaluatedRule {
            rule: "rate_limit",
            result: "skip",
            detail: Cow::Borrowed("Skipped — already denied"),
        });
        return;
    }

    let (count, limit) = rate_limit::check(ctx.session_id, ctx.user_id);

    if count >= limit {
        *denied = true;
        *deny_reason = Cow::Owned(format!("Rate limit exceeded: {count}/{limit} calls this minute"));
        *deny_policy = Cow::Borrowed("rate_limit");
        rules.push(EvaluatedRule {
            rule: "rate_limit",
            result: "fail",
            detail: Cow::Owned(format!("{count}/{limit} calls this minute — limit exceeded")),
        });
    } else {
        rules.push(EvaluatedRule {
            rule: "rate_limit",
            result: "pass",
            detail: Cow::Owned(format!("{count}/{limit} calls this minute")),
        });
    }
}
