//! Loading a tab as people, and shaping each person for the template.
//!
//! This file holds the shared shaping and the bridge tab; the three
//! credential tabs are in `credentials.rs` and use the same helpers.
//! Every loader reads twice: a page of people with their totals, then the
//! items belonging to the people on that page, stitched back under them in
//! order. A read that fails degrades to an empty tab rather than an error
//! page: the KPI strip above it comes from different statements and is still
//! worth showing, and a fleet page that refuses to render because one of four
//! tables was slow is worse than one that says a tab is empty.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::format::{format_token_total, short_num};
use crate::repositories::devices::sessions::{
    self, BridgeHostRow, BridgeUserSessionsRow, SessionQuery,
};

use super::context::{BridgeHostRowView, CellView, UserGroupView};

pub(super) const DATE: &str = "sp-table__cell--date";
pub(super) const NUM: &str = "sp-table__cell--num sp-u-num";

pub(super) fn stamp(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M").to_string()
}

pub(super) fn maybe_stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(|| "—".to_owned(), stamp)
}

pub(super) const fn cell(value: String, class: &'static str) -> CellView {
    CellView { value, class }
}

pub(super) fn plural(n: i64, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

pub(super) fn ids(users: impl Iterator<Item = UserId>) -> Vec<String> {
    users.map(|id| id.as_str().to_owned()).collect()
}

// Why: the person's row is built once and the tab fills in the rest. The
// four tabs agree on who a person is and disagree on everything else, so
// the shared part is exactly the part that is shared.
pub(super) fn group(user_id: &UserId, user_name: &str, tab: &str) -> UserGroupView {
    let id = user_id.as_str();
    UserGroupView {
        user_name: user_name.to_owned(),
        user_url: format!("/admin/users/{}", urlencoding::encode(id)),
        detail_id: format!("devices-{tab}-{}", urlencoding::encode(id)),
        count_label: String::new(),
        summary: String::new(),
        cells: Vec::new(),
        status_label: "",
        status_tone: "",
        sessions: Vec::new(),
        pats: Vec::new(),
        certs: Vec::new(),
        links: Vec::new(),
    }
}

pub(super) fn credential_summary(total: i64, active: i64) -> String {
    let inactive = total - active;
    if inactive == 0 {
        "all active".to_owned()
    } else if active == 0 {
        "none active".to_owned()
    } else {
        format!("{active} active · {inactive} not")
    }
}

// Why: revoked outranks expired. A revoked credential that later passes its
// own expiry is still revoked, and labelling it "Expired" would read as
// having lapsed on its own rather than having been taken away.
pub(super) const fn credential_state(revoked: bool, expired: bool) -> (&'static str, &'static str) {
    if revoked {
        ("Revoked", "muted")
    } else if expired {
        ("Expired", "warn")
    } else {
        ("Active", "ok")
    }
}

// Why: a person's badge is their best credential. One live token among four
// revoked ones is a person who can still get in, and "Revoked" on that row
// would be read as the opposite.
pub(super) const fn rollup_state(active: i64, any_revoked: bool) -> (&'static str, &'static str) {
    if active > 0 {
        ("Active", "ok")
    } else if any_revoked {
        ("Revoked", "muted")
    } else {
        ("Expired", "warn")
    }
}

pub(super) async fn load_sessions(
    pool: &PgPool,
    query: SessionQuery<'_>,
) -> (Vec<UserGroupView>, i64) {
    let (users, total) = sessions::list_bridge_users_paged(pool, query)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "bridge user listing failed"))
        .unwrap_or_default();
    let user_ids = ids(users.iter().map(|u| u.user_id.clone()));
    let hosts = sessions::list_bridge_hosts_for_users(pool, &user_ids, query.stale_only)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "bridge host listing failed"))
        .unwrap_or_default();
    let groups = users
        .iter()
        .map(|user| session_group(user, &hosts))
        .collect();
    (groups, total)
}

fn session_group(user: &BridgeUserSessionsRow, hosts: &[BridgeHostRow]) -> UserGroupView {
    let mut view = group(&user.user_id, &user.user_name, "bridges");
    view.sessions = hosts
        .iter()
        .filter(|h| h.user_id == user.user_id)
        .map(host_row)
        .collect();
    view.count_label = plural(user.hosts, "host");
    view.summary = view
        .sessions
        .iter()
        .map(|h| h.hostname.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    view.cells = vec![
        cell(user.latest_version.clone(), "sp-col-version sp-u-mono"),
        cell(stamp(user.first_started_at), DATE),
        cell(stamp(user.last_heartbeat_at), DATE),
        cell(short_num(user.forwarded_total), NUM),
        cell(format_token_total(user.tokens_total), NUM),
    ];
    (view.status_label, view.status_tone) = if user.any_active {
        ("Active", "ok")
    } else {
        ("Stale", "warn")
    };
    view
}

fn host_row(row: &BridgeHostRow) -> BridgeHostRowView {
    BridgeHostRowView {
        hostname: row.hostname.clone(),
        os: row.os.clone(),
        version: row.bridge_version.clone(),
        started_display: stamp(row.started_at),
        heartbeat_display: stamp(row.last_heartbeat_at),
        forwarded_display: short_num(row.forwarded_total),
        tokens_display: format_token_total(row.tokens_total),
        sessions_display: (row.session_count > 1).then(|| plural(row.session_count, "session")),
        status_label: if row.is_stale { "Stale" } else { "Active" },
        status_tone: if row.is_stale { "warn" } else { "ok" },
    }
}
