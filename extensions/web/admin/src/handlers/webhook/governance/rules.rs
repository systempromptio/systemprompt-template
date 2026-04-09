use super::rate_limit;
use super::secrets::detect_secrets;
use super::types::{EvaluatedRule, GovernanceContext, RuleEvaluation};

const ADMIN_ONLY_TOOL_PREFIXES: &[&str] = &["mcp__systemprompt__", "mcp__skill-manager__"];

pub(super) fn evaluate(ctx: &GovernanceContext<'_>) -> RuleEvaluation {
    let mut rules = Vec::new();
    let mut denied = false;
    let mut deny_reason = String::new();
    let mut deny_policy = String::new();

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
            decision: "deny",
            reason: deny_reason,
            policy: deny_policy,
            rules,
        }
    } else {
        RuleEvaluation {
            decision: "allow",
            reason: "All governance rules passed".to_string(),
            policy: "default_allow".to_string(),
            rules,
        }
    }
}

fn evaluate_secret_detection(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut String,
    deny_policy: &mut String,
) {
    match detect_secrets(ctx.tool_input) {
        Some((pattern_name, redacted)) => {
            *denied = true;
            *deny_reason = format!(
                "SECURITY BREACH: Plaintext secret detected in tool input \
                 — {pattern_name} ({redacted})"
            );
            *deny_policy = "secret_injection".to_string();
            rules.push(EvaluatedRule {
                rule: "secret_detection",
                result: "fail",
                detail: format!(
                    "Plaintext secret detected: {pattern_name} — matched '{redacted}' in tool_input"
                ),
            });
        }
        None => {
            rules.push(EvaluatedRule {
                rule: "secret_detection",
                result: "pass",
                detail: "No plaintext secrets detected in tool input".to_string(),
            });
        }
    }
}

fn evaluate_scope(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut String,
    deny_policy: &mut String,
) {
    if *denied {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "skip",
            detail: "Skipped — already denied".to_string(),
        });
        return;
    }

    if ctx.agent_scope == "admin" {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "pass",
            detail: "admin scope grants unrestricted tool access".to_string(),
        });
        return;
    }

    if ctx.agent_scope == "unknown" {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "warn",
            detail: "Agent scope could not be resolved, treating as restricted".to_string(),
        });
        return;
    }

    let requires_admin = ADMIN_ONLY_TOOL_PREFIXES
        .iter()
        .any(|prefix| ctx.tool_name.starts_with(prefix));

    if requires_admin {
        *denied = true;
        *deny_reason = format!(
            "{} scope cannot access admin-only tool: {}",
            ctx.agent_scope, ctx.tool_name
        );
        *deny_policy = "scope_restriction".to_string();
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "fail",
            detail: format!(
                "{} scope cannot access tools matching admin-only prefixes",
                ctx.agent_scope
            ),
        });
    } else {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "pass",
            detail: format!(
                "{} scope is allowed for tool: {}",
                ctx.agent_scope, ctx.tool_name
            ),
        });
    }
}

fn evaluate_blocklist(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut String,
    deny_policy: &mut String,
) {
    if *denied {
        rules.push(EvaluatedRule {
            rule: "tool_blocklist",
            result: "skip",
            detail: "Skipped — already denied".to_string(),
        });
        return;
    }

    let is_destructive = ctx.tool_name.contains("delete")
        || ctx.tool_name.contains("drop")
        || ctx.tool_name.contains("destroy");

    if is_destructive && ctx.agent_scope != "admin" {
        *denied = true;
        *deny_reason = format!(
            "Destructive tool '{}' blocked for {} scope",
            ctx.tool_name, ctx.agent_scope
        );
        *deny_policy = "tool_blocklist".to_string();
        rules.push(EvaluatedRule {
            rule: "tool_blocklist",
            result: "fail",
            detail: format!(
                "Tool '{}' matches destructive pattern blocklist",
                ctx.tool_name
            ),
        });
    } else {
        rules.push(EvaluatedRule {
            rule: "tool_blocklist",
            result: "pass",
            detail: "Tool not on restricted list".to_string(),
        });
    }
}

fn evaluate_rate_limit(
    ctx: &GovernanceContext<'_>,
    rules: &mut Vec<EvaluatedRule>,
    denied: &mut bool,
    deny_reason: &mut String,
    deny_policy: &mut String,
) {
    if *denied {
        rules.push(EvaluatedRule {
            rule: "rate_limit",
            result: "skip",
            detail: "Skipped — already denied".to_string(),
        });
        return;
    }

    let (count, limit) = rate_limit::check(ctx.session_id, ctx.user_id);

    if count >= limit {
        *denied = true;
        *deny_reason = format!("Rate limit exceeded: {count}/{limit} calls this minute");
        *deny_policy = "rate_limit".to_string();
        rules.push(EvaluatedRule {
            rule: "rate_limit",
            result: "fail",
            detail: format!("{count}/{limit} calls this minute — limit exceeded"),
        });
    } else {
        rules.push(EvaluatedRule {
            rule: "rate_limit",
            result: "pass",
            detail: format!("{count}/{limit} calls this minute"),
        });
    }
}
