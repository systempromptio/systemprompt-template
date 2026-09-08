//! Assembling the `/admin/mcp` list: joining declaration to runtime, deriving
//! status, and ordering.
//!
//! The join is a full outer one in spirit. Every declared server appears even
//! with no traffic, and every server the runtime tables name appears even when
//! nothing declares it — the second case is an operator problem (a binary
//! serving under a name the catalog does not know) and hiding it would be the
//! one failure this page must not have.

use std::collections::HashMap;

use crate::handlers::catalog::sorting::apply_direction;
use crate::handlers::ssr::format::format_duration_ms;
use crate::repositories::mcp::runtime::{McpProxyIdentityCount, McpServerActivity};
use crate::repositories::overview::liveness::{
    HEARTBEAT_INTERVAL_SECS, McpHeartbeatRow, liveness_state,
};
use crate::types::McpServerDetail;

use super::view::McpServerRow;

pub(super) const BASE_URL: &str = "/admin/mcp";
pub(super) const WINDOW_HOURS: i64 = 24;

// Why: the runtime facts for one server, keyed by the name the runtime used.
// Three sources, deliberately separate: the heartbeat comes from the overview
// repository the dashboard also reads, the identities from the MCP repository,
// and the traffic from the executions table.
pub(super) struct Runtime {
    pub heartbeat: HashMap<String, McpHeartbeatRow>,
    pub identities: HashMap<String, McpProxyIdentityCount>,
    pub activity: HashMap<String, McpServerActivity>,
}

impl Runtime {
    pub(super) fn new(
        heartbeat: Vec<McpHeartbeatRow>,
        identities: Vec<McpProxyIdentityCount>,
        activity: Vec<McpServerActivity>,
    ) -> Self {
        Self {
            heartbeat: heartbeat
                .into_iter()
                .map(|h| (h.server_id.clone(), h))
                .collect(),
            identities: identities
                .into_iter()
                .map(|i| (i.server_id.clone(), i))
                .collect(),
            activity: activity
                .into_iter()
                .map(|a| (a.server_name.clone(), a))
                .collect(),
        }
    }

    // Why: every name any runtime table knows, so a server serving under a
    // name the catalog never declared is still listed rather than dropped.
    pub(super) fn names(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .heartbeat
            .keys()
            .chain(self.identities.keys())
            .chain(self.activity.keys())
            .cloned()
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

// Why: the declaration decides the first two answers and the heartbeat decides
// the rest. A server nothing declares, or one declared and switched off, is not
// a liveness question at all — asking the heartbeat about it would report
// "No sessions" for a server that is off on purpose.
//
// The liveness third is `overview::liveness`'s rule verbatim, interval and all,
// so the dashboard strip and this table cannot disagree about the same server.
fn status_of(
    configured: bool,
    enabled: bool,
    heartbeat: Option<chrono::DateTime<chrono::Utc>>,
) -> (&'static str, &'static str) {
    if !configured {
        return ("Unconfigured", "warn");
    }
    if !enabled {
        return ("Disabled", "muted");
    }
    let state = liveness_state(chrono::Utc::now(), heartbeat, HEARTBEAT_INTERVAL_SECS);
    (state.label(), state.tone())
}

// Why: one place turns two counts into a percentage, so the display rounding
// and the precision-loss reasoning are stated once rather than at each site.
#[expect(
    clippy::cast_precision_loss,
    reason = "call counts are far below the f64 mantissa; this value is only displayed"
)]
fn percentage(part: i64, whole: i64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    (part as f64 / whole as f64) * 100.0
}

pub(super) fn error_rate(calls: i64, errors: i64) -> (String, &'static str) {
    if calls == 0 {
        return ("\u{2014}".to_owned(), "muted");
    }
    let pct = percentage(errors, calls);
    let tone = if pct >= 10.0 {
        "err"
    } else if pct > 0.0 {
        "warn"
    } else {
        "ok"
    };
    (format!("{pct:.1}%"), tone)
}

fn delta(calls: i64, prior: i64) -> (String, &'static str) {
    if prior == 0 {
        return (String::new(), "");
    }
    let pct = percentage(calls - prior, prior);
    if pct.abs() < 0.5 {
        return ("0%".to_owned(), "");
    }
    let dir = if pct > 0.0 { "up" } else { "down" };
    (format!("{pct:+.0}%"), dir)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "an averaged millisecond figure is rendered to whole milliseconds"
)]
const fn whole_ms(value: f64) -> i64 {
    value.round() as i64
}

fn ms_display(ms: Option<f64>) -> String {
    ms.map_or_else(
        || "\u{2014}".to_owned(),
        |v| format_duration_ms(whole_ms(v)),
    )
}

fn when_display(t: Option<chrono::DateTime<chrono::Utc>>) -> String {
    t.map_or_else(
        || "\u{2014}".to_owned(),
        crate::handlers::ssr::format::local_time,
    )
}

fn auth_label(server: Option<&McpServerDetail>) -> String {
    let Some(server) = server else {
        return "unknown".to_owned();
    };
    if !server.oauth_required {
        return "open".to_owned();
    }
    let audience = if server.oauth_audience.is_empty() {
        "any"
    } else {
        server.oauth_audience.as_str()
    };
    if server.oauth_scopes.is_empty() {
        return format!("aud {audience}");
    }
    format!("aud {audience} / {}", server.oauth_scopes.join(" "))
}

fn transport_of(server: Option<&McpServerDetail>) -> String {
    match server {
        None => "\u{2014}".to_owned(),
        Some(s) if !s.endpoint.is_empty() => s.endpoint.clone(),
        Some(s) => format!("localhost:{}", s.port),
    }
}

pub(super) struct RowInputs<'a> {
    pub id: &'a str,
    pub server: Option<&'a McpServerDetail>,
    pub runtime: &'a Runtime,
    pub plugin_count: usize,
    pub assignment_count: i64,
}

pub(super) fn build_row(input: &RowInputs<'_>) -> McpServerRow {
    let heartbeat = input.runtime.heartbeat.get(input.id);
    let identities = input.runtime.identities.get(input.id);
    let activity = input.runtime.activity.get(input.id);
    let configured = input.server.is_some();
    let enabled = input.server.is_some_and(|s| s.enabled);
    let last_heartbeat = heartbeat.and_then(|h| h.last_heartbeat);
    let (status_label, status_tone) = status_of(configured, enabled, last_heartbeat);

    let calls = activity.map_or(0, |a| a.calls);
    let errors = activity.map_or(0, |a| a.failures + a.timeouts);
    let (error_rate_display, error_tone) = error_rate(calls, errors);
    let (delta_display, delta_dir) = delta(calls, activity.map_or(0, |a| a.prior_calls));

    McpServerRow {
        id: input.id.to_owned(),
        description: input
            .server
            .map(|s| s.description.clone())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| "Not declared in services/mcp.".to_owned()),
        detail_url: format!("{BASE_URL}/{}", input.id),
        matrix_url: super::super::view::matrix_url(crate::types::ENTITY_MCP_SERVER, input.id),
        source_path: input
            .server
            .map(|s| s.source_path.clone())
            .unwrap_or_default(),
        configured,
        enabled,
        status_label,
        status_tone,
        server_type: input
            .server
            .map_or_else(|| "\u{2014}".to_owned(), |s| s.server_type.clone()),
        transport: transport_of(input.server),
        auth_label: auth_label(input.server),
        oauth_required: input.server.is_some_and(|s| s.oauth_required),
        sessions_open: heartbeat.map_or(0, |h| h.active_sessions),
        alive: liveness_state(chrono::Utc::now(), last_heartbeat, HEARTBEAT_INTERVAL_SECS)
            .is_alive(),
        last_heartbeat_display: when_display(last_heartbeat),
        proxy_identities: identities.map_or(0, |i| i.identities),
        calls,
        errors,
        error_rate_display,
        error_tone,
        p95_display: ms_display(activity.and_then(|a| a.p95_ms)),
        distinct_users: activity.map_or(0, |a| a.distinct_users),
        delta_display,
        delta_dir,
        last_call_display: when_display(activity.and_then(|a| a.last_call_at)),
        plugin_count: input.plugin_count,
        assignment_count: input.assignment_count,
    }
}

// Why: Order the assembled rows by the requested column.
pub(super) fn sort_rows(rows: &mut [McpServerRow], key: &str, dir: &str) {
    match key {
        "status" => apply_direction(rows, dir, |a, b| {
            a.alive.cmp(&b.alive).then_with(|| a.id.cmp(&b.id))
        }),
        "sessions" => apply_direction(rows, dir, |a, b| a.sessions_open.cmp(&b.sessions_open)),
        "identities" => {
            apply_direction(rows, dir, |a, b| {
                a.proxy_identities.cmp(&b.proxy_identities)
            });
        },
        "errors" => apply_direction(rows, dir, |a, b| a.errors.cmp(&b.errors)),
        "last" => apply_direction(rows, dir, |a, b| {
            a.last_call_display.cmp(&b.last_call_display)
        }),
        "grants" => apply_direction(rows, dir, |a, b| {
            a.assignment_count.cmp(&b.assignment_count)
        }),
        "id" => apply_direction(rows, dir, |a, b| a.id.cmp(&b.id)),
        _ => apply_direction(rows, dir, |a, b| a.calls.cmp(&b.calls)),
    }
}
