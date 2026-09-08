//! Row shaping for the secrets audit trail.

use serde::Serialize;

use crate::handlers::ssr::format::local_time;
use crate::repositories::governance::secret_audit_log::SecretAuditRow;

#[derive(Debug, Serialize)]
pub(super) struct SecretAuditRowView {
    pub(super) created_at: String,
    pub(super) action: String,
    pub(super) tone: &'static str,
    pub(super) var_name: String,
    pub(super) plugin_id: String,
    pub(super) owner: String,
    pub(super) owner_url: String,
    pub(super) actor: String,
    pub(super) actor_url: String,
    pub(super) third_party: bool,
    pub(super) ip_address: String,
}

// Why: `accessed` is toned as a warning and `deleted` as an error, not because
// either is wrong, but because those are the two rows an auditor scans for. A
// creation or an update is the system working.
const fn tone_of(action: &str) -> &'static str {
    match action.as_bytes() {
        b"accessed" => "warn",
        b"deleted" => "err",
        b"rotated" => "ok",
        _ => "muted",
    }
}

pub(super) fn rows(rows: &[SecretAuditRow]) -> Vec<SecretAuditRowView> {
    rows.iter()
        .map(|r| SecretAuditRowView {
            created_at: local_time(r.created_at),
            tone: tone_of(&r.action),
            action: r.action.clone(),
            var_name: r.var_name.clone(),
            plugin_id: r.plugin_id.clone(),
            owner: r.user_id.as_str().to_owned(),
            owner_url: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
            actor: r.actor_id.as_str().to_owned(),
            actor_url: format!("/admin/users/{}", urlencoding::encode(r.actor_id.as_str())),
            third_party: r.actor_id != r.user_id,
            ip_address: r
                .ip_address
                .clone()
                .unwrap_or_else(|| "\u{2014}".to_owned()),
        })
        .collect()
}
