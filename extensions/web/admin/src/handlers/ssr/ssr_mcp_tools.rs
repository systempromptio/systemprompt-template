use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

#[derive(Debug)]
struct RecentToolExecution {
    tool_name: String,
    status: String,
    user_id: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug)]
struct ToolByAgent {
    agent_id: String,
    tool_name: String,
    usage_count: i64,
}

/// SSR page for `/admin/mcp/tools` — tool availability per agent, execution
/// logs and tool-usage analytics backing `demo/mcp/03-mcp-tool-execution.sh`.
pub async fn mcp_tools_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let (success_rates_res, errors_res, recent_res, by_agent_res) = tokio::join!(
        repositories::dashboard_queries::fetch_tool_success_rates(&pool),
        repositories::dashboard_queries::fetch_recent_mcp_errors(&pool),
        fetch_recent_tool_executions(&pool),
        fetch_tools_by_agent(&pool),
    );

    let success_rates = success_rates_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch tool success rates");
        vec![]
    });
    let errors = errors_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch recent MCP errors");
        vec![]
    });
    let recent = recent_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch recent tool executions");
        vec![]
    });
    let by_agent = by_agent_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch tools by agent");
        vec![]
    });

    // JSON: template context for Handlebars rendering
    let success_json: Vec<serde_json::Value> = success_rates
        .iter()
        .map(|r| {
            json!({
                "tool_name": r.tool_name,
                "total": r.total,
                "successes": r.successes,
                "failures": r.failures,
                "success_pct": r.success_pct,
            })
        })
        .collect();

    let errors_json: Vec<serde_json::Value> = errors
        .iter()
        .map(|e| {
            json!({
                "tool_name": e.tool_name,
                "created_at": e.created_at,
            })
        })
        .collect();

    let recent_json: Vec<serde_json::Value> = recent
        .iter()
        .map(|r| {
            json!({
                "tool_name": r.tool_name,
                "status": r.status,
                "is_failed": r.status == "failed",
                "user_id": r.user_id,
                "created_at": r.created_at,
            })
        })
        .collect();

    let by_agent_json: Vec<serde_json::Value> = by_agent
        .iter()
        .map(|r| {
            json!({
                "agent_id": r.agent_id,
                "tool_name": r.tool_name,
                "usage_count": r.usage_count,
            })
        })
        .collect();

    let total_calls: i64 = success_rates.iter().map(|r| r.total).sum();
    let total_failures: i64 = success_rates.iter().map(|r| r.failures).sum();

    let data = json!({
        "page": "mcp-tools",
        "title": "MCP Tool Execution",
        "cli_command": "systemprompt infra logs tools list --limit 10",
        "demo_script": "demo/mcp/03-mcp-tool-execution.sh",
        "page_stats": [
            {"key": "calls", "value": total_calls, "label": "Tool calls"},
            {"key": "failures", "value": total_failures, "label": "Failures"},
            {"key": "tools", "value": success_rates.len(), "label": "Distinct tools"},
        ],
        "success_rates": success_json,
        "has_success_rates": !success_json.is_empty(),
        "recent_errors": errors_json,
        "has_recent_errors": !errors_json.is_empty(),
        "recent_executions": recent_json,
        "has_recent_executions": !recent_json.is_empty(),
        "tools_by_agent": by_agent_json,
        "has_tools_by_agent": !by_agent_json.is_empty(),
    });
    super::render_page(&engine, "mcp-tools", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_recent_tool_executions(
    pool: &PgPool,
) -> Result<Vec<RecentToolExecution>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT tool_name, status, user_id, created_at
           FROM mcp_tool_executions
           ORDER BY created_at DESC
           LIMIT 50"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RecentToolExecution {
            tool_name: r.tool_name,
            status: r.status,
            user_id: r.user_id,
            created_at: r.created_at,
        })
        .collect())
}

async fn fetch_tools_by_agent(pool: &PgPool) -> Result<Vec<ToolByAgent>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT COALESCE(agent_id, 'unknown') AS "agent_id!",
                  COALESCE(tool_name, 'unknown') AS "tool_name!",
                  COUNT(*)::BIGINT AS "usage_count!"
           FROM governance_decisions
           WHERE agent_id IS NOT NULL AND decision = 'allow'
           GROUP BY agent_id, tool_name
           ORDER BY COUNT(*) DESC
           LIMIT 30"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ToolByAgent {
            agent_id: r.agent_id,
            tool_name: r.tool_name,
            usage_count: r.usage_count,
        })
        .collect())
}
