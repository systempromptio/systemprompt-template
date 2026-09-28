//! Database → `policies.yaml`: the rows rendered as the file the loader
//! reads.
//!
//! Every monthly window goes out as its sentinel, never the day's live
//! value, so the exported file is the declaration and not a snapshot of
//! the rewrite. The header is regenerated with the export date; the long
//! operational commentary in the committed file is the operator's to keep
//! when they merge the result.

use chrono::{DateTime, Utc};
use systemprompt::ai::{GatewayPolicyConfig, GatewayPolicyEntry};

use super::month_window::{MONTH_WINDOW_SECONDS, normalise_spec};
use super::rows::PolicyRow;

fn header(exported_at: DateTime<Utc>) -> String {
    format!(
        "# Gateway policy baseline. Seeds `ai_gateway_policies` when the table is empty\n\
         # at boot (the governance_bootstrap job); otherwise the table is edited live at\n\
         # /admin/gateway/policies and /admin/sync compares the two and applies a\n\
         # direction on request.\n\
         #\n\
         # Exported from the database on {} by the console. A quota window with\n\
         # `window_seconds: {MONTH_WINDOW_SECONDS}` is a calendar month: the daily\n\
         # quota_month_window job keeps the live row aligned to the month end.\n\
         #\n\
         # Spec fields (systemprompt_ai::GatewayPolicySpec):\n\
         #   quota_mode, quota_windows, safety {{ mode, scanners, block_categories,\n\
         #                                       block_response_categories, history }}\n\n",
        exported_at.format("%Y-%m-%d")
    )
}

#[must_use]
pub fn render_policies_export(rows: &[PolicyRow], exported_at: DateTime<Utc>) -> String {
    let cfg = GatewayPolicyConfig {
        policies: rows
            .iter()
            .map(|r| GatewayPolicyEntry {
                name: r.name.clone(),
                enabled: r.enabled,
                priority: r.priority,
                spec: normalise_spec(&r.spec),
            })
            .collect(),
    };
    // Why: discard-ok: the config is built from rows that already parsed as
    // the same types, so serialising cannot fail; an empty body is the
    // visible sign if it somehow does
    let body = serde_yaml::to_string(&cfg).unwrap_or_default();
    format!("{}{body}", header(exported_at))
}
