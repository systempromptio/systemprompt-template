//! The column set `/admin/mcp` can be ordered by, and the tiles above it.
//!
//! Kept beside the row assembly rather than inside it: the columns are the
//! page's contract with the sort links and the template header row, and the
//! tiles are the same six numbers on the list page and on one server's detail
//! page, which is what lets a single server be read against the fleet.

use crate::handlers::catalog::sorting::SortColumn;
use crate::handlers::ssr::format::short_num;

use super::rows::error_rate;
use super::view::{McpKpiView, McpServerRow};

// Why: The columns `/admin/mcp` can be ordered by.
pub(super) fn columns() -> Vec<SortColumn> {
    vec![
        SortColumn {
            key: "id",
            label: "Server",
            class: "",
            hint: "The id declared in services/mcp, or the name the runtime used",
        },
        SortColumn {
            key: "status",
            label: "Status",
            class: "",
            hint: "Alive when a session has spoken inside the heartbeat window",
        },
        SortColumn {
            key: "sessions",
            label: "Sessions",
            class: "sp-table__cell--num",
            hint: "Open sessions, and how many of them are still beating",
        },
        SortColumn {
            key: "identities",
            label: "Ident",
            class: "sp-table__cell--num",
            hint: "Unexpired proxy identities a live session is acting as",
        },
        SortColumn {
            key: "calls",
            label: "Calls",
            class: "sp-table__cell--num",
            hint: "Tool executions in the last 24 hours, against the 24 before",
        },
        SortColumn {
            key: "errors",
            label: "Errors",
            class: "sp-table__cell--num",
            hint: "Failed and timed-out calls as a share of the window",
        },
        SortColumn {
            key: "last",
            label: "Last",
            class: "sp-table__cell--date",
            hint: "When this server last executed a tool",
        },
        SortColumn {
            key: "grants",
            label: "Grants",
            class: "sp-table__cell--num",
            hint: "Access-control rules naming this server",
        },
    ]
}

// Why: The five headline facts, plus the two that only matter when they are
// wrong.
pub(super) fn kpis(rows: &[McpServerRow]) -> Vec<McpKpiView> {
    let configured = rows.iter().filter(|r| r.configured).count();
    let alive = rows.iter().filter(|r| r.alive).count();
    let sessions: i64 = rows.iter().map(|r| r.sessions_open).sum();
    let identities: i64 = rows.iter().map(|r| r.proxy_identities).sum();
    let calls: i64 = rows.iter().map(|r| r.calls).sum();
    let errors: i64 = rows.iter().map(|r| r.errors).sum();
    let unconfigured = rows.len() - configured;
    let (rate, rate_tone) = error_rate(calls, errors);

    vec![
        McpKpiView {
            label: "Declared",
            value: configured.to_string(),
            sub: format!("{unconfigured} serving undeclared"),
            tone: if unconfigured > 0 { "warn" } else { "" },
            unit: "",
        },
        McpKpiView {
            label: "Alive now",
            value: alive.to_string(),
            sub: format!("of {configured} declared"),
            tone: if alive == 0 { "warn" } else { "ok" },
            unit: "",
        },
        McpKpiView {
            label: "Open sessions",
            value: short_num(sessions),
            sub: format!("{identities} proxy identities"),
            tone: "",
            unit: "",
        },
        McpKpiView {
            label: "Calls 24h",
            value: short_num(calls),
            sub: "tool executions".to_owned(),
            tone: "",
            unit: "",
        },
        McpKpiView {
            label: "Errors 24h",
            value: short_num(errors),
            sub: "failed or timed out".to_owned(),
            tone: if errors > 0 { "err" } else { "ok" },
            unit: "",
        },
        McpKpiView {
            label: "Error rate",
            value: rate,
            sub: "of calls in the window".to_owned(),
            tone: rate_tone,
            unit: "",
        },
    ]
}
