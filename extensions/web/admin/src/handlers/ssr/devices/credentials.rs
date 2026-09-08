//! The three credential tabs — tokens, certificates, enrolment links — as
//! people.
//!
//! Each loader pages the holders, then reads the credentials of the holders
//! on that page and folds them beneath their owner. The shaping helpers and
//! the state rules (revoked beats expired; a person's badge is their best
//! credential) live in `data.rs`, so the bridge tab and these three cannot
//! drift apart on what a date or a status looks like.

use chrono::Utc;
use sqlx::PgPool;

use crate::repositories::devices::certs::{self, CertUserRow, FleetCertRow};
use crate::repositories::devices::links::{self, LinkUserRow, PendingLinkRow};
use crate::repositories::devices::pats::{self, ApiKeyUserRow, CredentialQuery, FleetApiKeyRow};

use super::context::{ApiKeyRowView, CertRowView, LinkRowView, UserGroupView};
use super::data::{
    DATE, cell, credential_state, credential_summary, group, ids, maybe_stamp, plural,
    rollup_state, stamp,
};

pub(super) async fn load_pats(
    pool: &PgPool,
    query: CredentialQuery<'_>,
    can_manage: bool,
) -> (Vec<UserGroupView>, i64) {
    let (users, total) = pats::list_api_key_users_paged(pool, query)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access token holder listing failed"))
        .unwrap_or_default();
    let user_ids = ids(users.iter().map(|u| u.user_id.clone()));
    let keys = pats::list_api_keys_for_users(pool, &user_ids, query.state)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access token listing failed"))
        .unwrap_or_default();
    let groups = users
        .iter()
        .map(|user| pat_group(user, &keys, can_manage))
        .collect();
    (groups, total)
}

fn pat_group(user: &ApiKeyUserRow, keys: &[FleetApiKeyRow], can_manage: bool) -> UserGroupView {
    let mut view = group(&user.user_id, &user.user_name, "pats");
    view.pats = keys
        .iter()
        .filter(|k| k.user_id == user.user_id)
        .map(|k| pat_row(k, can_manage))
        .collect();
    view.count_label = plural(user.total, "token");
    view.summary = credential_summary(user.total, user.active);
    view.cells = vec![
        cell(stamp(user.newest_created_at), DATE),
        cell(maybe_stamp(user.last_used_at), DATE),
        cell(
            user.next_expires_at
                .map_or_else(|| "Never".to_owned(), stamp),
            DATE,
        ),
    ];
    let any_revoked = view.pats.iter().any(|k| k.status_label == "Revoked");
    (view.status_label, view.status_tone) = rollup_state(user.active, any_revoked);
    view
}

fn pat_row(row: &FleetApiKeyRow, can_manage: bool) -> ApiKeyRowView {
    let expired = row.expires_at.is_some_and(|at| at < Utc::now());
    let (status_label, status_tone) = credential_state(row.revoked_at.is_some(), expired);
    ApiKeyRowView {
        id: row.id.clone(),
        name: row.name.clone(),
        key_prefix: row.key_prefix.clone(),
        created_display: stamp(row.created_at),
        used_display: maybe_stamp(row.last_used_at),
        expires_display: row.expires_at.map_or_else(|| "Never".to_owned(), stamp),
        status_label,
        status_tone,
        can_revoke: can_manage && row.revoked_at.is_none(),
    }
}

pub(super) async fn load_certs(
    pool: &PgPool,
    query: CredentialQuery<'_>,
    can_manage: bool,
) -> (Vec<UserGroupView>, i64) {
    let (users, total) = certs::list_device_cert_users_paged(pool, query)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "device certificate holder listing failed"))
        .unwrap_or_default();
    let user_ids = ids(users.iter().map(|u| u.user_id.clone()));
    let rows = certs::list_device_certs_for_users(pool, &user_ids, query.state)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "device certificate listing failed"))
        .unwrap_or_default();
    let groups = users
        .iter()
        .map(|user| cert_group(user, &rows, can_manage))
        .collect();
    (groups, total)
}

fn cert_group(user: &CertUserRow, rows: &[FleetCertRow], can_manage: bool) -> UserGroupView {
    let mut view = group(&user.user_id, &user.user_name, "certs");
    view.certs = rows
        .iter()
        .filter(|c| c.user_id == user.user_id)
        .map(|c| cert_row(c, can_manage))
        .collect();
    view.count_label = plural(user.total, "certificate");
    view.summary = credential_summary(user.total, user.active);
    view.cells = vec![cell(stamp(user.latest_enrolled_at), DATE)];
    (view.status_label, view.status_tone) = rollup_state(user.active, user.active < user.total);
    view
}

fn cert_row(row: &FleetCertRow, can_manage: bool) -> CertRowView {
    let (status_label, status_tone) = credential_state(row.revoked_at.is_some(), false);
    CertRowView {
        id: row.id.clone(),
        label: row.label.clone(),
        fingerprint: row.fingerprint.clone(),
        enrolled_display: stamp(row.enrolled_at),
        status_label,
        status_tone,
        can_revoke: can_manage && row.revoked_at.is_none(),
    }
}

pub(super) async fn load_links(
    pool: &PgPool,
    sort: &str,
    dir: &str,
    limit: i64,
    offset: i64,
) -> (Vec<UserGroupView>, i64) {
    let (users, total) = links::list_pending_link_users_paged(pool, sort, dir, limit, offset)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "pending link holder listing failed"))
        .unwrap_or_default();
    let user_ids = ids(users.iter().map(|u| u.user_id.clone()));
    let rows = links::list_pending_links_for_users(pool, &user_ids)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "pending link listing failed"))
        .unwrap_or_default();
    let groups = users.iter().map(|user| link_group(user, &rows)).collect();
    (groups, total)
}

fn link_group(user: &LinkUserRow, rows: &[PendingLinkRow]) -> UserGroupView {
    let mut view = group(&user.user_id, &user.user_name, "links");
    view.links = rows
        .iter()
        .filter(|l| l.user_id == user.user_id)
        .map(link_row)
        .collect();
    view.count_label = plural(user.total, "code");
    view.summary = if user.pending == 0 {
        "all expired".to_owned()
    } else {
        format!("{} waiting", user.pending)
    };
    view.cells = vec![
        cell(stamp(user.newest_created_at), DATE),
        cell(stamp(user.latest_expires_at), DATE),
    ];
    (view.status_label, view.status_tone) = if user.pending > 0 {
        ("Waiting", "accent")
    } else {
        ("Expired", "warn")
    };
    view
}

fn link_row(row: &PendingLinkRow) -> LinkRowView {
    LinkRowView {
        created_display: stamp(row.created_at),
        expires_display: stamp(row.expires_at),
        status_label: if row.is_expired { "Expired" } else { "Waiting" },
        status_tone: if row.is_expired { "warn" } else { "accent" },
    }
}
