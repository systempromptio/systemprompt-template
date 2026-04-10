use crate::repositories;
use serde_json::json;
use sqlx::PgPool;

pub(super) async fn inject_mcp_access_and_costs(pool: &PgPool, data: &mut serde_json::Value) {
    let (mcp_access, token_usage) = tokio::join!(
        repositories::dashboard_queries::fetch_mcp_access_events(pool),
        repositories::dashboard_queries::fetch_token_usage_by_user(pool),
    );

    let mcp_events = mcp_access.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch MCP access events for dashboard");
        vec![]
    });
    let mcp_json: Vec<serde_json::Value> = mcp_events
        .iter()
        .map(|r| {
            json!({
                "server_name": r.server_name,
                "action": r.action,
                "is_granted": r.action == "granted",
                "description": r.description,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let tokens = token_usage.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch token usage for dashboard");
        vec![]
    });
    let max_tokens: i64 = tokens
        .iter()
        .map(|t| t.input_tokens + t.output_tokens)
        .max()
        .unwrap_or(1)
        .max(1);
    let tokens_json: Vec<serde_json::Value> = tokens
        .iter()
        .map(|r| {
            let total = r.input_tokens + r.output_tokens;
            let pct = total.saturating_mul(100) / max_tokens;
            json!({
                "label": r.label,
                "input_tokens": r.input_tokens,
                "output_tokens": r.output_tokens,
                "total_tokens": total,
                "event_count": r.event_count,
                "pct": pct,
            })
        })
        .collect();

    if let Some(obj) = data.as_object_mut() {
        obj.insert("mcp_access_events".to_string(), json!(mcp_json));
        obj.insert(
            "has_mcp_access_events".to_string(),
            json!(!mcp_json.is_empty()),
        );
        obj.insert("token_usage".to_string(), json!(tokens_json));
        obj.insert(
            "has_token_usage".to_string(),
            json!(!tokens_json.is_empty()),
        );
    }
}

pub(super) async fn inject_governance_data(pool: &PgPool, data: &mut serde_json::Value) {
    let (gov_events, gov_counts) = tokio::join!(
        repositories::governance::fetch_governance_events(pool),
        repositories::governance::fetch_governance_counts(pool),
    );
    let gov_events = gov_events.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch governance events for dashboard");
        vec![]
    });
    let counts = gov_counts.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch governance counts for dashboard");
        repositories::governance::GovernanceCounts {
            total: 0,
            allowed: 0,
            denied: 0,
            secret_breaches: 0,
        }
    });
    let gov_json: Vec<serde_json::Value> = gov_events
        .iter()
        .map(|r| {
            json!({
                "user_id": r.user_id,
                "tool_name": r.tool_name,
                "agent_id": r.agent_id,
                "decision": r.decision,
                "is_denied": r.decision == "deny",
                "reason": r.reason,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    if let Some(obj) = data.as_object_mut() {
        obj.insert("governance_total".to_string(), json!(counts.total));
        obj.insert("governance_allowed".to_string(), json!(counts.allowed));
        obj.insert("governance_denied".to_string(), json!(counts.denied));
        obj.insert(
            "governance_secret_breaches".to_string(),
            json!(counts.secret_breaches),
        );
        obj.insert("governance_events".to_string(), json!(gov_json));
        obj.insert(
            "has_governance_events".to_string(),
            json!(!gov_json.is_empty()),
        );
    }
}
