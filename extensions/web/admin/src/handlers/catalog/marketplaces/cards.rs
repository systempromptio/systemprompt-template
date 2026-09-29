//! One card per marketplace: the manifest, who reaches it, and where it came
//! from.

use std::collections::HashMap;

use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_shared::GroupId;

use super::skills_of;
use super::view::{AudienceMatrixView, MarketplaceCardView, marketplace_url};
use crate::handlers::ssr::sync_plane::HashView;
use crate::repositories::marketplace::manifests::MarketplaceConfigSummary;
use crate::repositories::sync::marketplace_versions_db::list_current_marketplace_versions;
use crate::repositories::sync::provenance::{OwnedKind, Provenance, owned_by};
use crate::repositories::sync::sources::build_sources;

pub(super) async fn current_hashes(pool: &PgPool) -> HashMap<String, String> {
    // Why: discard-ok: a missing versions table leaves every card with no hash
    list_current_marketplace_versions(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: versions unreadable"))
        .unwrap_or_default()
        .into_iter()
        .map(|v| (v.marketplace_id.as_str().to_owned(), v.content_hash))
        .collect()
}

pub(super) struct CardInputs<'a> {
    pub manifests: &'a [MarketplaceConfigSummary],
    pub audience: &'a AudienceMatrixView,
    pub grants: &'a HashMap<MarketplaceId, Vec<GroupId>>,
    pub plugin_catalog: &'a [crate::types::PluginDetail],
    pub versions: &'a HashMap<String, String>,
}

pub(super) fn card_views(inputs: &CardInputs<'_>) -> Vec<MarketplaceCardView> {
    // Why: discard-ok: an unreadable profile leaves every card as base
    let sources = build_sources()
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: sources unreadable"))
        .ok();
    inputs
        .manifests
        .iter()
        .map(|m| {
            let provenance = sources.as_ref().map_or_else(Provenance::base, |s| {
                owned_by(s, OwnedKind::Marketplace, m.id.as_str())
            });
            let assigned_groups = inputs.grants.get(&m.id).cloned().unwrap_or_default();
            // Why: the resolved count, not the declared one. A group listed in
            // the manifest that a deny rule closes is not an audience, and the
            // two numbers side by side are how that shows up.
            let allowed_subjects = inputs
                .audience
                .rows
                .iter()
                .filter(|row| {
                    row.cells
                        .iter()
                        .any(|c| c.marketplace_id == m.id && c.is_allow)
                })
                .count();
            MarketplaceCardView {
                id: m.id.clone(),
                name: m.name.clone(),
                description: m.description.clone(),
                version: m.version.clone(),
                enabled: m.enabled,
                visibility: m.visibility.clone(),
                detail_url: marketplace_url(m.id.as_str()),
                roles: m.access.roles.clone(),
                groups: m.access.groups.clone(),
                projects: m.access.projects.clone(),
                plugin_count: m.plugins.len(),
                skill_count: skills_of(inputs.plugin_catalog, &m.plugins).len(),
                mcp_count: m.mcp_servers.len(),
                default_included: m.access.default_included,
                assigned_group_count: assigned_groups.len(),
                assigned_groups,
                allowed_subjects,
                source: provenance.source,
                source_tone: provenance.tone,
                content_hash: inputs
                    .versions
                    .get(m.id.as_str())
                    .and_then(|h| HashView::of(h)),
            }
        })
        .collect()
}
