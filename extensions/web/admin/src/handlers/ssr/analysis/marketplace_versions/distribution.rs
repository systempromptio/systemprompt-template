//! The Distribution view: what the publication pipeline delivered for this
//! marketplace's skills — the published head of each managed resource, the
//! bridge's delivery claims, the receipts devices sent back and how many
//! entitled devices hold the served publication. Withdrawal proposals are
//! decided here.

use std::collections::BTreeSet;
use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::marketplace::managed::{WithdrawalProposal, WithdrawalStatus};

use crate::error::AdminResult;
use crate::handlers::ssr::analysis::lifecycle::view::{
    DistributionView, PublicationView, ReceiptView, distribution_view, list_publication_views,
    receipt_view,
};
use crate::repositories::analysis::marketplace_versions::MarketplaceVersionMetricsRow;
use crate::repositories::analysis::publications::{
    list_publication_lifecycle, list_resource_coverage,
};
use crate::routes::managed_state::ManagedState;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoverageView {
    pub resource_key: String,
    pub eligible_devices: i64,
    pub acknowledged_devices: i64,
    pub verified_devices: i64,
    pub pct: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DistributionPage {
    pub publications: Vec<PublicationView>,
    pub withdrawals: Vec<WithdrawalProposal>,
    pub deliveries: Vec<DistributionView>,
    pub receipts: Vec<ReceiptView>,
    pub coverage: Vec<CoverageView>,
    pub published_resources: usize,
    pub pending_withdrawals: usize,
    pub failed_deliveries: usize,
    pub verified_receipts: usize,
    pub devices_current: i64,
    pub devices_eligible: i64,
}

fn normalise(key: &str) -> String {
    key.replace('-', "_")
}

// Why: the manifest names skills by config id; managed resources are keyed
// by the same id modulo dash/underscore, which the inventory sync already
// treats as one name. Without a manifest (a legacy version) every resource
// is shown, and the page says so.
fn skill_keys(rows: &[MarketplaceVersionMetricsRow]) -> Option<BTreeSet<String>> {
    let current = rows.iter().find(|r| r.effective_until.is_none())?;
    let manifest = current.manifest.as_ref()?;
    Some(
        manifest
            .0
            .plugins
            .iter()
            .flat_map(|p| p.skills.iter().map(|s| normalise(s.skill_id.as_str())))
            .collect(),
    )
}

pub(crate) async fn build(
    pool: &PgPool,
    managed: &Arc<ManagedState>,
    rows: &[MarketplaceVersionMetricsRow],
) -> AdminResult<DistributionPage> {
    let scope = skill_keys(rows);
    let in_scope = |key: &str| scope.as_ref().is_none_or(|s| s.contains(&normalise(key)));

    let lifecycle: Vec<_> = list_publication_lifecycle(pool, managed.owner.as_str())
        .await?
        .into_iter()
        .filter(|row| in_scope(&row.resource_key))
        .collect();
    let resource_ids: BTreeSet<String> = lifecycle.iter().map(|r| r.resource_id.clone()).collect();
    let publication_ids: BTreeSet<String> =
        lifecycle.iter().map(|r| r.publication_id.clone()).collect();
    let published_resources = resource_ids.len();

    let withdrawals: Vec<WithdrawalProposal> = managed
        .repository
        .list_withdrawal_proposals(&managed.owner)
        .await?
        .into_iter()
        .filter(|p| resource_ids.contains(p.resource_id.as_str()))
        .collect();
    let deliveries: Vec<DistributionView> = managed
        .repository
        .list_distribution_status(&managed.owner)
        .await?
        .into_iter()
        .filter(|d| publication_ids.contains(d.publication_id.as_str()))
        .map(distribution_view)
        .collect();
    let receipts: Vec<ReceiptView> = managed
        .repository
        .list_installation_receipts(&managed.owner, None)
        .await?
        .into_iter()
        .filter(|r| resource_ids.contains(r.resource_id.as_str()))
        .map(receipt_view)
        .collect();
    let coverage: Vec<CoverageView> = list_resource_coverage(pool, managed.owner.as_str())
        .await?
        .into_iter()
        .filter(|c| in_scope(&c.resource_key))
        .map(|c| {
            let body = c.body.0;
            CoverageView {
                resource_key: c.resource_key,
                eligible_devices: body.eligible_devices,
                acknowledged_devices: body.current_acknowledged_devices,
                verified_devices: body.current_verified_devices,
                pct: (body.eligible_devices > 0)
                    .then(|| body.current_acknowledged_devices * 100 / body.eligible_devices),
            }
        })
        .collect();

    Ok(DistributionPage {
        pending_withdrawals: withdrawals
            .iter()
            .filter(|p| p.status == WithdrawalStatus::Pending)
            .count(),
        failed_deliveries: deliveries.iter().filter(|d| d.status == "failed").count(),
        verified_receipts: receipts.iter().filter(|r| r.is_verified()).count(),
        devices_current: coverage.iter().map(|c| c.acknowledged_devices).sum(),
        devices_eligible: coverage.iter().map(|c| c.eligible_devices).sum(),
        published_resources,
        publications: list_publication_views(lifecycle),
        withdrawals,
        deliveries,
        receipts,
        coverage,
    })
}
