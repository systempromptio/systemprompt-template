use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::ProfileBootstrap;

use crate::admin::types::{GovernQuery, HookEventPayload};

use super::helpers::{extract_bearer_token, get_jwt_config};

// ---------------------------------------------------------------------------
// Governance response types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct GovernanceResponse {
    decision: &'static str,
    reason: String,
    policy: String,
    agent_scope: String,
    tool_name: String,
    evaluated_rules: Vec<EvaluatedRule>,
}

#[derive(Debug, Serialize)]
struct EvaluatedRule {
    rule: &'static str,
    result: &'static str,
    detail: String,
}

// ---------------------------------------------------------------------------
// Governance rules (hardcoded for demo clarity)
// ---------------------------------------------------------------------------

/// Tools that only `admin` scope agents may invoke.
const ADMIN_ONLY_TOOL_PREFIXES: &[&str] = &[
    "mcp__systemprompt__",
    "mcp__skill-manager__",
];

/// Maximum tool calls per session per minute before rate-limiting kicks in.
const RATE_LIMIT_PER_MINUTE: i64 = 60;

// ---------------------------------------------------------------------------
// Secret detection patterns
// ---------------------------------------------------------------------------

/// Each entry: (pattern_name, prefix/regex-like hint).
/// We check if any string value in tool_input contains these patterns.
const SECRET_PATTERNS: &[(&str, &str)] = &[
    ("AWS Access Key", "AKIA"),
    ("AWS Secret Key", "aws_secret_access_key"),
    ("GitHub Token (classic)", "ghp_"),
    ("GitHub Token (fine-grained)", "github_pat_"),
    ("GitHub OAuth", "gho_"),
    ("GitHub App User-to-Server", "ghu_"),
    ("GitHub App Server-to-Server", "ghs_"),
    ("GitHub App Refresh", "ghr_"),
    ("GitLab Token", "glpat-"),
    ("Slack Bot Token", "xoxb-"),
    ("Slack User Token", "xoxp-"),
    ("Slack Webhook", "hooks.slack.com/services/"),
    ("Stripe Secret Key", "sk_live_"),
    ("Stripe Restricted Key", "rk_live_"),
    ("Google API Key", "AIza"),
    ("Anthropic API Key", "sk-ant-"),
    ("OpenAI API Key", "sk-proj-"),
    ("Twilio Auth Token", "twilio_auth_token"),
    ("SendGrid API Key", "SG."),
    ("Mailgun API Key", "key-"),
    ("Heroku API Key", "heroku_api_key"),
    ("Private Key Header", "-----BEGIN RSA PRIVATE KEY-----"),
    ("Private Key Header (EC)", "-----BEGIN EC PRIVATE KEY-----"),
    ("Private Key Header (generic)", "-----BEGIN PRIVATE KEY-----"),
    ("Generic password field", "password="),
    ("Generic secret field", "secret="),
    ("Bearer token literal", "Bearer eyJ"),
    ("JWT token (raw)", "eyJhbGciOi"),
    ("Database URL with password", "postgresql://"),
    ("Database URL with password (mysql)", "mysql://"),
    ("MongoDB connection string", "mongodb+srv://"),
    ("Redis URL with auth", "redis://"),
];

/// Recursively extract all string values from a JSON value.
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

/// Scan all string values in tool_input for known secret patterns.
/// Returns None if clean, Some((pattern_name, redacted_match)) if a secret is found.
fn detect_secrets(tool_input: Option<&serde_json::Value>) -> Option<(String, String)> {
    let Some(input) = tool_input else {
        return None;
    };

    let mut strings = Vec::new();
    collect_strings(input, &mut strings);

    for s in &strings {
        for &(pattern_name, prefix) in SECRET_PATTERNS {
            if s.contains(prefix) {
                // Redact the match for the audit log — show first 8 chars max
                let match_start = s.find(prefix).unwrap_or(0);
                let snippet_end = (match_start + 12).min(s.len());
                let redacted = format!("{}...[REDACTED]", &s[match_start..snippet_end]);
                return Some((pattern_name.to_string(), redacted));
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Resolve agent scope from YAML config
// ---------------------------------------------------------------------------

fn resolve_agent_scope(agent_id: &str) -> String {
    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .ok();

    let Some(services_path) = services_path else {
        return "unknown".to_string();
    };

    let agents_dir = services_path.join("agents");
    if !agents_dir.exists() {
        return "unknown".to_string();
    }

    // Scan YAML files for the agent and extract its OAuth scope
    let Ok(entries) = std::fs::read_dir(&agents_dir) else {
        return "unknown".to_string();
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        if let Some(agents_map) = config.get("agents").and_then(|a| a.as_mapping()) {
            if let Some(agent_val) = agents_map.get(&serde_yaml::Value::String(agent_id.to_string()))
            {
                // Extract scope from oauth.scopes[0] or security[0].oauth2[0]
                if let Some(scopes) = agent_val
                    .get("oauth")
                    .and_then(|o| o.get("scopes"))
                    .and_then(|s| s.as_sequence())
                {
                    if let Some(first) = scopes.first().and_then(|s| s.as_str()) {
                        return first.to_string();
                    }
                }
                // Fallback: check card.security
                if let Some(security) = agent_val
                    .get("card")
                    .and_then(|c| c.get("security"))
                    .and_then(|s| s.as_sequence())
                {
                    for sec in security {
                        if let Some(oauth_scopes) =
                            sec.get("oauth2").and_then(|o| o.as_sequence())
                        {
                            if let Some(first) = oauth_scopes.first().and_then(|s| s.as_str()) {
                                return first.to_string();
                            }
                        }
                    }
                }
                return "unknown".to_string();
            }
        }
    }

    "unknown".to_string()
}

// ---------------------------------------------------------------------------
// Evaluate governance rules
// ---------------------------------------------------------------------------

struct RuleEvaluation {
    decision: &'static str,
    reason: String,
    policy: String,
    rules: Vec<EvaluatedRule>,
}

async fn evaluate_rules(
    pool: &PgPool,
    tool_name: &str,
    agent_scope: &str,
    session_id: &str,
    user_id: &str,
    tool_input: Option<&serde_json::Value>,
) -> RuleEvaluation {
    let mut rules = Vec::new();
    let mut denied = false;
    let mut deny_reason = String::new();
    let mut deny_policy = String::new();

    // -----------------------------------------------------------------------
    // Rule 0: Secret injection detection (ALWAYS runs, never skipped)
    // -----------------------------------------------------------------------
    match detect_secrets(tool_input) {
        Some((pattern_name, redacted)) => {
            denied = true;
            deny_reason = format!(
                "SECURITY BREACH: Plaintext secret detected in tool input — {pattern_name} ({redacted})"
            );
            deny_policy = "secret_injection".to_string();
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

    // -----------------------------------------------------------------------
    // Rule 1: Scope check
    // -----------------------------------------------------------------------
    if agent_scope == "admin" {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "pass",
            detail: "admin scope grants unrestricted tool access".to_string(),
        });
    } else if agent_scope == "unknown" {
        rules.push(EvaluatedRule {
            rule: "scope_check",
            result: "warn",
            detail: "Agent scope could not be resolved, treating as restricted".to_string(),
        });
    } else {
        // Non-admin: check if the tool requires admin scope
        let requires_admin = ADMIN_ONLY_TOOL_PREFIXES
            .iter()
            .any(|prefix| tool_name.starts_with(prefix));

        if requires_admin {
            denied = true;
            deny_reason = format!(
                "{agent_scope} scope cannot access admin-only tool: {tool_name}"
            );
            deny_policy = "scope_restriction".to_string();
            rules.push(EvaluatedRule {
                rule: "scope_check",
                result: "fail",
                detail: format!(
                    "{agent_scope} scope cannot access tools matching admin-only prefixes"
                ),
            });
        } else {
            rules.push(EvaluatedRule {
                rule: "scope_check",
                result: "pass",
                detail: format!("{agent_scope} scope is allowed for tool: {tool_name}"),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Rule 2: Tool blocklist (skip if already denied)
    // -----------------------------------------------------------------------
    if denied {
        rules.push(EvaluatedRule {
            rule: "tool_blocklist",
            result: "skip",
            detail: "Skipped — already denied by scope check".to_string(),
        });
    } else {
        // For demo: block destructive patterns for non-admin
        let is_destructive = tool_name.contains("delete")
            || tool_name.contains("drop")
            || tool_name.contains("destroy");

        if is_destructive && agent_scope != "admin" {
            denied = true;
            deny_reason = format!(
                "Destructive tool '{tool_name}' blocked for {agent_scope} scope"
            );
            deny_policy = "tool_blocklist".to_string();
            rules.push(EvaluatedRule {
                rule: "tool_blocklist",
                result: "fail",
                detail: format!("Tool '{tool_name}' matches destructive pattern blocklist"),
            });
        } else {
            rules.push(EvaluatedRule {
                rule: "tool_blocklist",
                result: "pass",
                detail: "Tool not on restricted list".to_string(),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Rule 3: Rate limit (skip if already denied)
    // -----------------------------------------------------------------------
    if denied {
        rules.push(EvaluatedRule {
            rule: "rate_limit",
            result: "skip",
            detail: "Skipped — already denied".to_string(),
        });
    } else {
        let count = count_recent_governance_calls(pool, session_id, user_id).await;
        if count >= RATE_LIMIT_PER_MINUTE {
            denied = true;
            deny_reason = format!(
                "Rate limit exceeded: {count}/{RATE_LIMIT_PER_MINUTE} calls this minute"
            );
            deny_policy = "rate_limit".to_string();
            rules.push(EvaluatedRule {
                rule: "rate_limit",
                result: "fail",
                detail: format!("{count}/{RATE_LIMIT_PER_MINUTE} calls this minute — limit exceeded"),
            });
        } else {
            rules.push(EvaluatedRule {
                rule: "rate_limit",
                result: "pass",
                detail: format!("{count}/{RATE_LIMIT_PER_MINUTE} calls this minute"),
            });
        }
    }

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

// ---------------------------------------------------------------------------
// Count recent tool calls for rate limiting
// ---------------------------------------------------------------------------

async fn count_recent_governance_calls(pool: &PgPool, session_id: &str, user_id: &str) -> i64 {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM governance_decisions
         WHERE session_id = $1 AND user_id = $2
         AND created_at > NOW() - INTERVAL '1 minute'",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(pool)
    .await;

    result.unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Record governance decision for audit
// ---------------------------------------------------------------------------

async fn record_governance_decision(
    pool: &PgPool,
    user_id: &str,
    session_id: &str,
    tool_name: &str,
    agent_id: Option<&str>,
    agent_scope: &str,
    decision: &str,
    policy: &str,
    reason: &str,
    evaluated_rules: &serde_json::Value,
    plugin_id: Option<&str>,
) {
    let id = uuid::Uuid::new_v4().to_string();
    let rules_json = evaluated_rules;

    let result = sqlx::query(
        "INSERT INTO governance_decisions
         (id, user_id, session_id, tool_name, agent_id, agent_scope, decision, policy, reason, evaluated_rules, plugin_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(session_id)
    .bind(tool_name)
    .bind(agent_id)
    .bind(agent_scope)
    .bind(decision)
    .bind(policy)
    .bind(reason)
    .bind(&rules_json)
    .bind(plugin_id)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::error!(error = %e, "Failed to record governance decision");
    }
}

// ---------------------------------------------------------------------------
// Handler: POST /api/public/hooks/govern
// ---------------------------------------------------------------------------

pub(crate) async fn govern_tool_use(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<GovernQuery>,
    Json(payload): Json<HookEventPayload>,
) -> Response {
    // --- Auth ---
    let Some(token) = extract_bearer_token(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization header"})),
        )
            .into_response();
    };

    let (jwt_secret, jwt_issuer) = match get_jwt_config() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal configuration error"})),
            )
                .into_response();
        }
    };

    let claims = match systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_secret,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
        ],
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Governance webhook JWT validation failed");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or expired token"})),
            )
                .into_response();
        }
    };

    // --- Extract fields ---
    let user_id = &claims.sub;
    let tool_name = payload.tool_name.as_deref().unwrap_or("unknown");
    let agent_id = payload.agent_id.as_deref();
    let session_id = payload.session_id.as_deref().unwrap_or("unknown");
    let plugin_id = query.plugin_id.as_deref();

    // --- Resolve agent scope ---
    let agent_scope = agent_id
        .map(|id| resolve_agent_scope(id))
        .unwrap_or_else(|| "unknown".to_string());

    // --- Evaluate governance rules ---
    let evaluation = evaluate_rules(
        &pool,
        tool_name,
        &agent_scope,
        session_id,
        user_id,
        payload.tool_input.as_ref(),
    )
    .await;

    // --- Record audit trail (async, don't block response) ---
    let p = pool.clone();
    let uid = user_id.to_string();
    let sid = session_id.to_string();
    let tn = tool_name.to_string();
    let aid = agent_id.map(str::to_string);
    let asc = agent_scope.clone();
    let dec = evaluation.decision.to_string();
    let pol = evaluation.policy.clone();
    let rea = evaluation.reason.clone();
    let rules_json = serde_json::to_value(&evaluation.rules).unwrap_or_default();
    let pid = plugin_id.map(str::to_string);
    let rules_json_clone = rules_json.clone();
    tokio::spawn(async move {
        record_governance_decision(
            &p,
            &uid,
            &sid,
            &tn,
            aid.as_deref(),
            &asc,
            &dec,
            &pol,
            &rea,
            &rules_json_clone,
            pid.as_deref(),
        )
        .await;
    });

    // --- Return decision ---
    let response = GovernanceResponse {
        decision: evaluation.decision,
        reason: evaluation.reason,
        policy: evaluation.policy,
        agent_scope,
        tool_name: tool_name.to_string(),
        evaluated_rules: evaluation.rules,
    };

    (StatusCode::OK, Json(response)).into_response()
}
