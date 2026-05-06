//! `/admin/governance/flow` — Live decision flow visualization.
//!
//! Renders a recent window of `governance_decisions` (last 5min by default)
//! as a ranked list of subjects + a timeline. The page subscribes to
//! `/admin/api/sse/audit` for new decisions and prepends them client-side.
//! Companion to the existing audit trail, oriented around motion rather than
//! tabular browsing.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const DEFAULT_REPLAY: &str = "last-5m";

#[derive(Debug, Deserialize)]
pub struct FlowQuery {
    pub replay: Option<String>,
}

pub async fn governance_flow_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<FlowQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let replay = query.replay.as_deref().unwrap_or(DEFAULT_REPLAY);
    let minutes = replay_minutes(replay);

    let rows = fetch_recent_decisions(&pool, minutes).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_recent_decisions failed");
        Vec::new()
    });

    let (top_denied_subjects, top_denied_entities) = aggregate_top(&rows);

    let initial_json = serde_json::to_string(&json!({ "rows": rows }))
        .unwrap_or_else(|_| "{\"rows\":[]}".to_string());

    let data = json!({
        "page": "governance-flow",
        "title": "Governance — Decision Flow",
        "replay": replay,
        "replay_options": replay_options(replay),
        "row_count": rows.len(),
        "initial_json": initial_json,
        "top_denied_subjects": top_denied_subjects,
        "top_denied_entities": top_denied_entities,
        "has_top_subjects": !top_denied_subjects_empty(&rows),
    });

    super::render_page(&engine, "governance-flow", &data, &user_ctx, &mkt_ctx)
}

fn replay_minutes(preset: &str) -> i32 {
    match preset {
        "last-1m" => 1,
        "last-15m" => 15,
        "last-1h" => 60,
        _ => 5,
    }
}

fn replay_options(active: &str) -> Vec<serde_json::Value> {
    [
        ("last-1m", "Last 1 minute"),
        ("last-5m", "Last 5 minutes"),
        ("last-15m", "Last 15 minutes"),
        ("last-1h", "Last hour"),
    ]
    .iter()
    .map(|(value, label)| {
        json!({
            "value": value,
            "label": label,
            "active": *value == active,
        })
    })
    .collect()
}

async fn fetch_recent_decisions(
    pool: &PgPool,
    minutes: i32,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let sql = "SELECT g.id, g.user_id, g.session_id, g.tool_name, g.agent_id, g.agent_scope, \
               g.decision, g.policy, g.reason, g.evaluated_rules, g.created_at, \
               u.username AS subject_username, u.department AS subject_department \
               FROM governance_decisions g \
               LEFT JOIN users u ON u.id = g.user_id \
               WHERE g.created_at > NOW() - ($1::int || ' minutes')::interval \
               ORDER BY g.created_at DESC \
               LIMIT 500";
    let rows = sqlx::query(sql).bind(minutes).fetch_all(pool).await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("id")?;
        let user_id: String = row.try_get("user_id")?;
        let session_id: String = row.try_get("session_id")?;
        let tool_name: String = row.try_get("tool_name")?;
        let agent_id: Option<String> = row.try_get("agent_id").ok();
        let agent_scope: Option<String> = row.try_get("agent_scope").ok();
        let decision: String = row.try_get("decision")?;
        let policy: String = row.try_get("policy")?;
        let reason: String = row.try_get("reason")?;
        let evaluated_rules: Option<serde_json::Value> = row.try_get("evaluated_rules").ok();
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
        let subject_username: Option<String> = row.try_get("subject_username").ok();
        let subject_department: Option<String> = row.try_get("subject_department").ok();

        let subject_label = subject_username.as_deref().map_or_else(
            || user_id.clone(),
            |u| match subject_department.as_deref() {
                Some(d) if !d.is_empty() => format!("{u} ({d})"),
                _ => u.to_string(),
            },
        );

        out.push(json!({
            "id": id,
            "user_id": user_id,
            "session_id": session_id,
            "tool_name": tool_name,
            "agent_id": agent_id,
            "agent_scope": agent_scope,
            "decision": decision,
            "policy": policy,
            "reason": reason,
            "evaluated_rules": evaluated_rules,
            "created_at": created_at.to_rfc3339(),
            "subject_label": subject_label,
            "subject_department": subject_department,
        }));
    }
    Ok(out)
}

fn aggregate_top(
    rows: &[serde_json::Value],
) -> (Vec<serde_json::Value>, Vec<serde_json::Value>) {
    use std::collections::HashMap;
    let mut by_subject: HashMap<String, i64> = HashMap::new();
    let mut by_entity: HashMap<String, i64> = HashMap::new();
    for r in rows {
        if r.get("decision").and_then(|d| d.as_str()) != Some("deny") {
            continue;
        }
        if let Some(s) = r.get("subject_label").and_then(|x| x.as_str()) {
            *by_subject.entry(s.to_string()).or_insert(0) += 1;
        }
        if let Some(t) = r.get("tool_name").and_then(|x| x.as_str()) {
            *by_entity.entry(t.to_string()).or_insert(0) += 1;
        }
    }
    let to_top = |m: HashMap<String, i64>| -> Vec<serde_json::Value> {
        let mut v: Vec<(String, i64)> = m.into_iter().collect();
        v.sort_by_key(|item| std::cmp::Reverse(item.1));
        v.into_iter()
            .take(8)
            .map(|(label, count)| json!({ "label": label, "count": count }))
            .collect()
    };
    (to_top(by_subject), to_top(by_entity))
}

fn top_denied_subjects_empty(rows: &[serde_json::Value]) -> bool {
    !rows
        .iter()
        .any(|r| r.get("decision").and_then(|d| d.as_str()) == Some("deny"))
}
