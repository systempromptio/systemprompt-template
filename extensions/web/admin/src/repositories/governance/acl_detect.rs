//! Data access for the after-the-fact gateway-ACL detector.
//!
//! The detector re-runs the ACL resolver over recent `/v1/messages` requests
//! and records a `governance_decisions` row for any that should have been
//! denied. Both the scan query and the decision insert live here so the
//! handler depends on named repository methods rather than inline SQL.

use serde_json::Value;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, sqlx::FromRow)]
pub struct RecentAiRequest {
    pub id: String,
    pub user_id: UserId,
    pub session_id: Option<SessionId>,
    pub model: String,
}

#[derive(Debug)]
pub struct GatewayAclDecision<'a> {
    pub decision_id: &'a str,
    pub user_id: &'a str,
    pub session_id: &'a str,
    pub model: &'a str,
    pub agent_scope: &'a str,
    pub decision: &'a str,
    pub reason: &'a str,
    pub evaluated_rules: &'a Value,
}

pub async fn list_recent_unrejected_requests(
    pool: &PgPool,
    since_minutes: i64,
) -> Result<Vec<RecentAiRequest>, sqlx::Error> {
    sqlx::query_as!(
        RecentAiRequest,
        r#"SELECT id AS "id!", user_id AS "user_id!: UserId", session_id AS "session_id: SessionId", model AS "model!"
           FROM ai_requests
           WHERE created_at >= NOW() - ($1 || ' minutes')::interval
             AND status NOT IN ('rejected', 'denied')"#,
        since_minutes.to_string()
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_gateway_acl_decision(
    pool: &PgPool,
    decision: GatewayAclDecision<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO governance_decisions \
         (id, user_id, session_id, tool_name, agent_id, agent_scope, \
          decision, policy, reason, evaluated_rules, plugin_id) \
         VALUES ($1, $2, $3, $4, NULL, $5, $6, 'gateway_acl', $7, $8, NULL)",
        decision.decision_id,
        decision.user_id,
        decision.session_id,
        decision.model,
        decision.agent_scope,
        decision.decision,
        decision.reason,
        decision.evaluated_rules,
    )
    .execute(pool)
    .await?;
    Ok(())
}
