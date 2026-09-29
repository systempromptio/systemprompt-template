//! Row shaping for the Distribution view of a marketplace's versions.
//!
//! Identifiers and digests are rendered short with the full value in a
//! `title`, so a table of sixty-four-character hashes stays readable without
//! losing the value an operator needs to copy.

use serde::Serialize;
use systemprompt::marketplace::managed::{
    DistributionState, DistributionStatus, InstallationReceipt,
};

use crate::repositories::analysis::publications::PublicationLifecycleRow;

const SHORT: usize = 8;

fn short(value: &str) -> String {
    value.chars().take(SHORT).collect()
}

fn action_label(action: &str) -> (&'static str, &'static str) {
    match action {
        "initial_adoption" => ("Initial adoption", "info"),
        "publish_improvement" => ("Improvement", "ok"),
        "withdraw" => ("Withdrawn", "warn"),
        "rollback" => ("Rollback", "err"),
        _ => ("Reviewed", "muted"),
    }
}

fn distribution_tone(status: &str) -> &'static str {
    match status {
        "distributed" => "ok",
        "claimed" => "info",
        "failed" => "err",
        _ => "muted",
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct PublicationView {
    resource_key: String,
    resource_href: Option<String>,
    resource_full: String,
    generation: i64,
    action_label: &'static str,
    action_tone: &'static str,
    revision_short: Option<String>,
    revision_full: Option<String>,
    digest_short: Option<String>,
    digest_full: Option<String>,
    distribution_status: String,
    distribution_tone: &'static str,
    receipts: i64,
    reviewer_short: String,
    reviewer_full: String,
    limitations: String,
    provenance_short: Option<String>,
    provenance_full: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReceiptView {
    id_short: String,
    id_full: String,
    installation_short: String,
    installation_full: String,
    consumer_id: Option<String>,
    device_short: Option<String>,
    device_full: Option<String>,
    host: Option<String>,
    generation: i64,
    digest_short: String,
    digest_full: String,
    fully_verified: bool,
    verified_tone: &'static str,
    verified_label: &'static str,
    verified_at: chrono::DateTime<chrono::Utc>,
    installed_manifest: String,
    evidence: String,
}

impl ReceiptView {
    pub(crate) const fn is_verified(&self) -> bool {
        self.fully_verified
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct DistributionView {
    pub(crate) publication_short: String,
    pub(crate) publication_full: String,
    pub(crate) generation: i64,
    pub(crate) status: String,
    pub(crate) status_tone: &'static str,
    pub(crate) claimed_at: chrono::DateTime<chrono::Utc>,
    pub(crate) error: Option<String>,
}

pub(crate) fn list_publication_views(rows: Vec<PublicationLifecycleRow>) -> Vec<PublicationView> {
    rows.into_iter()
        .map(|row| {
            let (action_label, action_tone) = action_label(&row.action);
            PublicationView {
                resource_key: row.resource_key,
                resource_href: row
                    .revision_id
                    .as_deref()
                    .map(|id| format!("/admin/analysis/revisions/{id}")),
                resource_full: row.resource_id,
                generation: row.generation,
                action_label,
                action_tone,
                revision_short: row.revision_id.as_deref().map(short),
                revision_full: row.revision_id,
                digest_short: row.bundle_digest.as_deref().map(short),
                digest_full: row.bundle_digest,
                distribution_tone: distribution_tone(&row.distribution_status),
                distribution_status: row.distribution_status,
                receipts: row.installation_receipts,
                reviewer_short: short(&row.reviewer_id),
                reviewer_full: row.reviewer_id,
                limitations: row.limitations,
                provenance_short: row.provenance_hash.as_deref().map(short),
                provenance_full: row.provenance_hash,
            }
        })
        .collect()
}

pub(crate) fn receipt_view(receipt: InstallationReceipt) -> ReceiptView {
    let evidence = receipt.consumer_evidence.as_ref().map_or_else(
        || {
            receipt.client_evidence.as_ref().map_or_else(
                || "No retained evidence for this receipt.".to_owned(),
                |legacy| {
                    serde_json::to_string_pretty(legacy).unwrap_or_else(|_error| "{}".to_owned())
                },
            )
        },
        |consumer| serde_json::to_string_pretty(consumer).unwrap_or_else(|_error| "{}".to_owned()),
    );
    let (verified_tone, verified_label) = if receipt.fully_verified {
        ("ok", "Verified")
    } else {
        ("warn", "Partial")
    };
    ReceiptView {
        id_short: short(receipt.id.as_str()),
        id_full: receipt.id.as_str().to_owned(),
        installation_short: short(receipt.installation_id.as_str()),
        installation_full: receipt.installation_id.as_str().to_owned(),
        consumer_id: receipt.consumer_id.map(|id| id.as_str().to_owned()),
        device_short: receipt.device_id.as_ref().map(|id| short(id.as_str())),
        device_full: receipt.device_id.map(|id| id.as_str().to_owned()),
        host: receipt.host,
        generation: receipt.generation,
        digest_short: short(receipt.bundle_digest.as_str()),
        digest_full: receipt.bundle_digest.as_str().to_owned(),
        fully_verified: receipt.fully_verified,
        verified_tone,
        verified_label,
        verified_at: receipt.verified_at,
        installed_manifest: serde_json::to_string_pretty(&receipt.installed_manifest)
            .unwrap_or_else(|_error| "[]".to_owned()),
        evidence,
    }
}

pub(crate) fn distribution_view(row: DistributionStatus) -> DistributionView {
    let status = match row.status {
        DistributionState::Claimed => "claimed",
        DistributionState::Distributed => "distributed",
        DistributionState::Failed => "failed",
    };
    DistributionView {
        publication_short: short(row.publication_id.as_str()),
        publication_full: row.publication_id.as_str().to_owned(),
        generation: row.generation,
        status: status.to_owned(),
        status_tone: distribution_tone(status),
        claimed_at: row.claimed_at,
        error: row.error,
    }
}
