//! The Sources panel: where declarations come from, with a content hash for
//! each, assembled in-process from what core already records.
//!
//! `base` is this repository's `services/` tree. Its hash is computed the
//! way `systemprompt core services bundle` computes a bundle's
//! `content_hash` — the same file walk, the same digest — so the number on
//! the page equals the one on the base bundle published at this release.
//! Each `bundle:<name>` is a source the profile pins; its digest, version,
//! hash and fetch time come from the bundle cache's `state.json`, what it
//! owns from its cached `bundle.json`, and its provenance from the active
//! services root core installed at boot. Nothing here fetches.

use std::path::Path;

use serde::Serialize;
use systemprompt::config::ProfileBootstrap;
use systemprompt::loader::ConfigLoader;
use systemprompt::loader::bundle::{BundleCache, cache_root};
use systemprompt::loader::services_root::{
    ActiveServicesRoot, ServicesProvenance, ServicesRootBootstrap,
};
use systemprompt::models::profile::{Profile, ServicesSource};
use systemprompt::models::services::bundle::{
    BundleOwnership, ServicesBundleManifest, ServicesBundleState,
};

use super::source_badges::{BadgeInputs, SourceBadge, source_badges};
use super::tree_hash::base_tree_hash;

use crate::error::AdminResult;

#[derive(Debug, Clone, Serialize)]
pub struct BaseSourceView {
    pub version: &'static str,
    pub tree_hash: Option<String>,
    pub tree_path: String,
    pub provenance: &'static str,
    pub provenance_error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OwnsView {
    pub marketplaces: Vec<String>,
    pub plugins: Vec<String>,
    pub skills: Vec<String>,
    pub summary: String,
}

/// One pinned bundle. `owner_key` is `bundle:<name>` — the value `owner:`
/// takes in rules.yaml and the `source` stamped on rows a kit's own access
/// block wrote; `pin_command` is the recipe that re-pins it.
#[derive(Debug, Clone, Serialize)]
pub struct BundleSourceView {
    pub name: String,
    pub owner_key: String,
    pub transport: &'static str,
    pub reference: String,
    pub pinned_digest: Option<String>,
    pub following_tag: bool,
    pub active_digest: Option<String>,
    pub version: Option<String>,
    pub content_hash: Option<String>,
    pub fetched_at: Option<String>,
    pub owns: OwnsView,
    pub badges: Vec<SourceBadge>,
    pub pin_command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourcesView {
    pub base: BaseSourceView,
    pub bundles: Vec<BundleSourceView>,
    pub composed_hash: Option<String>,
    pub last_reconciled_hash: Option<String>,
    pub restart_pending: bool,
    pub on_fetch_failure: String,
    pub refresh_hint: String,
}

pub fn build_sources() -> AdminResult<SourcesView> {
    let profile = ProfileBootstrap::get()?;
    let active = ServicesRootBootstrap::get();
    let cache = BundleCache::new(cache_root(profile));
    let state = cache.read_state();
    let (provenance, provenance_error, composed_hash) = provenance_of(active);

    let restart_pending = !state.composed_hash.is_empty()
        && state.last_reconciled_hash.as_deref() != Some(state.composed_hash.as_str());

    let composed_marketplaces_declaring_access = marketplaces_declaring_access();

    let bundles = profile
        .services
        .sources
        .iter()
        .map(|source| {
            bundle_view(
                source,
                &state,
                &cache,
                restart_pending,
                &composed_marketplaces_declaring_access,
            )
        })
        .collect();

    Ok(SourcesView {
        base: BaseSourceView {
            version: env!("CARGO_PKG_VERSION"),
            tree_hash: base_tree_hash(Path::new(&profile.paths.services)),
            tree_path: profile.paths.services.clone(),
            provenance,
            provenance_error,
        },
        bundles,
        // Why: the boot-time provenance is a static; after an in-place import
        // the cache state is what names the composition being served.
        composed_hash: (!state.composed_hash.is_empty())
            .then(|| state.composed_hash.clone())
            .or(composed_hash),
        last_reconciled_hash: state.last_reconciled_hash,
        restart_pending,
        on_fetch_failure: serde_yaml::to_string(&profile.services.on_fetch_failure)
            .map(|s| s.trim().to_owned())
            .unwrap_or_default(),
        refresh_hint: refresh_hint(profile),
    })
}

fn provenance_of(
    active: Option<&ActiveServicesRoot>,
) -> (&'static str, Option<String>, Option<String>) {
    match active.map(|a| &a.provenance) {
        None | Some(ServicesProvenance::Bundled) => ("bundled", None, None),
        Some(ServicesProvenance::Fetched { composed_hash, .. }) => {
            ("fetched", None, Some(composed_hash.clone()))
        },
        Some(ServicesProvenance::LastGood {
            composed_hash,
            error,
        }) => (
            "last_good",
            Some(error.clone()),
            Some(composed_hash.clone()),
        ),
        Some(ServicesProvenance::BundledFallback { error }) => {
            ("bundled_fallback", Some(error.clone()), None)
        },
    }
}

fn bundle_view(
    source: &ServicesSource,
    state: &ServicesBundleState,
    cache: &BundleCache,
    restart_pending: bool,
    declaring_access: &[String],
) -> BundleSourceView {
    let (transport, reference) = source.oci.as_ref().map_or_else(
        || {
            (
                "https",
                source
                    .https
                    .as_ref()
                    .map(|h| h.url.clone())
                    .unwrap_or_default(),
            )
        },
        |o| ("oci", o.reference.clone()),
    );
    let pinned_digest = reference
        .split_once("@sha256:")
        .map(|(_, d)| format!("sha256:{d}"));
    let following_tag = transport == "oci" && pinned_digest.is_none();

    let active = state.sources.get(&source.name);
    let owns = active
        .and_then(|a| cache.read_manifest(&source.name, &a.content_hash).ok())
        .map(|m| owns_view(&m.manifest))
        .unwrap_or_default();

    let badges = source_badges(&BadgeInputs {
        name: &source.name,
        pinned_digest: pinned_digest.as_deref(),
        active_digest: active.map(|a| a.digest.as_str()),
        following_tag,
        restart_pending,
        kit_access: owns
            .marketplaces
            .iter()
            .filter(|m| declaring_access.contains(m))
            .map(String::as_str)
            .collect(),
    });

    let owner_key = format!("bundle:{}", source.name);
    let pin_command = format!(
        "just services-pin {} <digest>   # rewrites services.sources[{}] in the profile",
        source.name, source.name
    );
    BundleSourceView {
        name: source.name.clone(),
        owner_key,
        transport,
        reference,
        pinned_digest,
        following_tag,
        active_digest: active.map(|a| a.digest.clone()),
        version: active.map(|a| a.version.clone()),
        content_hash: active.map(|a| a.content_hash.clone()),
        fetched_at: active.map(|a| a.fetched_at.to_rfc3339()),
        owns,
        badges,
        pin_command,
    }
}

fn owns_view(manifest: &ServicesBundleManifest) -> OwnsView {
    let BundleOwnership {
        marketplaces,
        plugins,
        skills,
        ..
    } = &manifest.owns;
    OwnsView {
        summary: format!(
            "{} marketplace(s) · {} plugin(s) · {} skill(s)",
            marketplaces.len(),
            plugins.len(),
            skills.len()
        ),
        marketplaces: marketplaces.clone(),
        plugins: plugins.clone(),
        skills: skills.clone(),
    }
}

// Why: a kit that ships its own `access:` is a second truth for who reaches
// it. Core still ingests it at reconcile (source `bundle:<name>`), so the
// panel names it and the access-control drift shows those rows as bundle
// orphans that an overwrite deletes.
fn marketplaces_declaring_access() -> Vec<String> {
    ConfigLoader::load().map_or_else(
        |_| Vec::new(),
        |services| {
            services
                .marketplaces
                .iter()
                .filter(|(_, m)| m.access.declares_rules() || m.access.default_included)
                .map(|(id, _)| id.as_str().to_owned())
                .collect()
        },
    )
}

fn refresh_hint(profile: &Profile) -> String {
    format!(
        "curl -X POST -H \"Authorization: Bearer $ADMIN_TOKEN\" \
         http://127.0.0.1:{}/api/v1/admin/services/refresh",
        profile.server.port
    )
}
