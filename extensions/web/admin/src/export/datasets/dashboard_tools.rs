//! `/admin/analytics` — the tables behind the dashboard's Skills and Tools
//! tabs, which read the MCP member's executions.

use async_trait::async_trait;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::repositories::analytics::site::skills::list_skill_stats;
use crate::repositories::analytics::site::tools::{list_tool_servers, list_tool_stats};

use super::dashboard::{live_dataset, plan};

pub(crate) struct SkillStats;
pub(crate) struct ToolStats;
pub(crate) struct ToolServers;

const SKILL_COLUMNS: &[Column] = &[
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("resource_id", "Resource", CellKind::Text).optional(),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new("slash_invocations", "Slash invocations", CellKind::Integer),
    Column::new("tool_invocations", "Tool invocations", CellKind::Integer),
    Column::new("attributed_invocations", "Attributed", CellKind::Integer),
    Column::new("distinct_users", "Users", CellKind::Integer),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("priced_requests", "Priced requests", CellKind::Integer).optional(),
    Column::new(
        "conversation_cost_usd",
        "Conversation cost (USD)",
        CellKind::Money,
    ),
    Column::new(
        "conversation_tokens",
        "Conversation tokens",
        CellKind::Integer,
    ),
];

live_dataset!(
    SkillStats,
    "analytics-skills",
    "Skills",
    "One row per skill: invocations by kind, people, conversations and requests over the window.",
    SKILL_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let (rows, total) = list_skill_stats(ctx.pool, p.range, &p.scope, ctx.limit, 0).await?;
        Ok(Table {
            rows: rows
                .iter()
                .map(|r| {
                    vec![
                        r.skill.as_str().into(),
                        Cell::opt_text(r.resource_id.as_deref()),
                        r.invocations.into(),
                        r.slash_invocations.into(),
                        r.tool_invocations.into(),
                        r.attributed_invocations.into(),
                        r.distinct_users.into(),
                        r.conversations.into(),
                        r.requests.into(),
                        r.priced_requests.into(),
                        Cell::Money(r.conversation_cost_microdollars),
                        r.conversation_tokens.into(),
                    ]
                })
                .collect(),
            total,
        })
    }
);

const TOOL_COLUMNS: &[Column] = &[
    Column::new("server", "Server", CellKind::Text),
    Column::new("tool", "Tool", CellKind::Text),
    Column::new("executions", "Executions", CellKind::Integer),
    Column::new("succeeded", "Succeeded", CellKind::Integer),
    Column::new("failed", "Failed", CellKind::Integer),
    Column::new("pending", "Pending", CellKind::Integer).optional(),
    Column::new("p50_ms", "p50 (ms)", CellKind::Decimal),
    Column::new("p95_ms", "p95 (ms)", CellKind::Decimal),
    Column::new("distinct_users", "Users", CellKind::Integer),
];

live_dataset!(
    ToolStats,
    "analytics-tools",
    "Tools",
    "One row per server and tool: executions, outcomes, latency and people over the window.",
    TOOL_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let (rows, total) = list_tool_stats(ctx.pool, p.range, &p.scope, ctx.limit, 0).await?;
        Ok(Table {
            rows: rows
                .iter()
                .map(|r| {
                    vec![
                        r.server_name.as_str().into(),
                        r.tool_name.as_str().into(),
                        r.executions.into(),
                        r.succeeded.into(),
                        r.failed.into(),
                        r.pending.into(),
                        Cell::opt_decimal(r.p50_ms),
                        Cell::opt_decimal(r.p95_ms),
                        r.distinct_users.into(),
                    ]
                })
                .collect(),
            total,
        })
    }
);

const SERVER_COLUMNS: &[Column] = &[
    Column::new("server", "Server", CellKind::Text),
    Column::new("executions", "Executions", CellKind::Integer),
    Column::new("succeeded", "Succeeded", CellKind::Integer),
    Column::new("tools", "Tools", CellKind::Integer),
    Column::new("distinct_users", "Users", CellKind::Integer),
];

live_dataset!(
    ToolServers,
    "analytics-tool-servers",
    "Tool servers",
    "One row per tool server: executions, successes, tools used and people over the window.",
    SERVER_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let rows = list_tool_servers(ctx.pool, p.range, &p.scope).await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.server_name.as_str().into(),
                        r.executions.into(),
                        r.succeeded.into(),
                        r.tools.into(),
                        r.distinct_users.into(),
                    ]
                })
                .collect(),
        ))
    }
);
