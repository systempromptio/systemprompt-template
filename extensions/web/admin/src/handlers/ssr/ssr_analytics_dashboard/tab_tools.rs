//! View assembly for the Tools tab.
//!
//! Everything here is measured: `mcp_tool_executions` records one row per real
//! call with its own status and duration. Pending rows carry no duration, so
//! they are counted in their own column rather than folded into the
//! percentile as zeros.

use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::site::tools::{ToolServerRow, ToolStatsRow};

use super::context::{KpiTile, ToolRowView, ToolServerRowView, ToolsTabView};
use super::tab_models::{ms, pct, share};
use super::view::compact;
use super::{AnalyticsDashboardQuery, PAGE_SIZE, urls};

pub(super) fn tools_tab(
    servers: &[ToolServerRow],
    rows: &[ToolStatsRow],
    total_rows: i64,
    input: (i64, &AnalyticsDashboardQuery),
) -> ToolsTabView {
    let (page, query) = input;
    let server_max = servers.iter().map(|s| s.executions).max().unwrap_or(0);
    let tool_max = rows.iter().map(|r| r.executions).max().unwrap_or(0);

    let server_views: Vec<ToolServerRowView> = servers
        .iter()
        .map(|s| ToolServerRowView {
            executions: s.executions,
            share_pct: share(s.executions, server_max),
            success_display: format!("{:.0}%", pct(s.succeeded, s.executions)),
            tools: s.tools,
            users: s.distinct_users,
            drill_url: format!("/admin/mcp/{}", urlencoding::encode(&s.server_name)),
            server_name: s.server_name.clone(),
        })
        .collect();

    let tool_views: Vec<ToolRowView> = rows.iter().map(|r| tool_row(r, tool_max, query)).collect();

    let pagination = (total_rows > PAGE_SIZE).then(|| {
        urls::build_pagination(
            query,
            PageWindow::new(
                page,
                PAGE_SIZE,
                total_rows,
                i64::try_from(rows.len()).unwrap_or(PAGE_SIZE),
                "tools",
            ),
        )
    });

    ToolsTabView {
        kpis: kpis(servers, rows),
        server_count: servers.len(),
        tool_count: total_rows,
        has_servers: !server_views.is_empty(),
        servers: server_views,
        has_rows: !tool_views.is_empty(),
        rows: tool_views,
        pagination,
    }
}

fn tool_row(r: &ToolStatsRow, max: i64, query: &AnalyticsDashboardQuery) -> ToolRowView {
    // Why: pending calls have not succeeded or failed yet, so the rate is over
    // what has settled — counting them as failures would make a slow tool look
    // like a broken one.
    let settled = r.succeeded + r.failed;
    let success = pct(r.succeeded, settled);
    ToolRowView {
        executions: r.executions,
        share_pct: share(r.executions, max),
        success_display: if settled == 0 {
            "—".to_owned()
        } else {
            format!("{success:.0}%")
        },
        success_tone: if settled == 0 {
            "muted"
        } else if success >= 95.0 {
            "ok"
        } else if success >= 80.0 {
            "warn"
        } else {
            "err"
        },
        failed: r.failed,
        pending_display: if r.pending == 0 {
            "—".to_owned()
        } else {
            r.pending.to_string()
        },
        p50_display: ms(r.p50_ms),
        p95_display: ms(r.p95_ms),
        users: r.distinct_users,
        drill_url: urls::drill_url(query, "tool", &r.tool_name),
        tool_name: r.tool_name.clone(),
        server_name: r.server_name.clone(),
    }
}

fn kpis(servers: &[ToolServerRow], rows: &[ToolStatsRow]) -> Vec<KpiTile> {
    let executions: i64 = servers.iter().map(|s| s.executions).sum();
    let succeeded: i64 = servers.iter().map(|s| s.succeeded).sum();
    let users = servers.iter().map(|s| s.distinct_users).max().unwrap_or(0);
    let slowest = rows
        .iter()
        .filter_map(|r| r.p95_ms.map(|v| (r.tool_name.clone(), v)))
        .max_by(|a, b| a.1.total_cmp(&b.1));
    let rate = pct(succeeded, executions);
    vec![
        KpiTile {
            label: "Tool calls".to_owned(),
            value: compact(executions),
            sub: format!("{} tools across {} servers", rows.len(), servers.len()),
            tone: "accent",
        },
        KpiTile {
            label: "Success rate".to_owned(),
            value: format!("{rate:.0}%"),
            sub: format!("{succeeded} of {executions} settled successfully"),
            tone: if rate >= 95.0 {
                "ok"
            } else if rate >= 80.0 {
                "warn"
            } else {
                "err"
            },
        },
        KpiTile {
            label: "Slowest tool (p95)".to_owned(),
            value: slowest
                .as_ref()
                .map_or_else(|| "—".to_owned(), |(_, v)| ms(Some(*v))),
            sub: slowest.map_or_else(|| "no timed calls".to_owned(), |(name, _)| name),
            tone: "warn",
        },
        KpiTile {
            label: "Callers".to_owned(),
            value: users.to_string(),
            sub: "distinct users on the busiest server".to_owned(),
            tone: "ok",
        },
    ]
}
