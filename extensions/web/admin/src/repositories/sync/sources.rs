//! The Sources panel: where declarations come from, with a content hash for
//! each, assembled in-process from what core already records.
//!
//! `base` is this repository's `services/` tree. Its hash is computed the
//! way `systemprompt core services bundle` computes a bundle's
//! `content_hash` — the same file walk, the same digest — so the number on
//! the page equals the one on the base bundle published at this release.
//! Each `bundle:<name>` is a source the profile pins, read from the install
//! services view as its name, transport and credential-free location. The
//! bundle cache state (active digest, version, content hash, fetch time,
//! what a bundle owns) and the active root's provenance are not readable by
//! an install member, so the panel shows the pins without them. Nothing here
//! fetches.

use crate::env::AdminEnv;

use std::path::Path;

use serde::Serialize;
use systemprompt::manifest::services::{ServicesSourceView, SourceTransport};

use super::source_badges::{BadgeInputs, SourceBadge, source_badges};
use super::tree_hash::base_tree_hash;


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

pub fn build_sources(env: &AdminEnv) -> SourcesView {
    let declaring_access = marketplaces_declaring_access(env);
    let bundles = env
        .services_view()
        .sources()
        .iter()
        .map(|source| bundle_view(source, &declaring_access))
        .collect();
    let baked = env.baked_services_root();
    SourcesView {
        base: BaseSourceView {
            version: env!("CARGO_PKG_VERSION"),
            tree_hash: base_tree_hash(Path::new(&baked)),
            tree_path: baked.display().to_string(),
            provenance: "unreported",
            provenance_error: None,
        },
        bundles,
        composed_hash: None,
        last_reconciled_hash: None,
        restart_pending: false,
        on_fetch_failure: String::new(),
        refresh_hint: refresh_hint(env),
    }
}

fn bundle_view(source: &ServicesSourceView, declaring_access: &[String]) -> BundleSourceView {
    let transport = match source.transport {
        SourceTransport::Https => "https",
        SourceTransport::Oci => "oci",
    };
    let reference = source.location.clone().unwrap_or_default();
    let pinned_digest = reference
        .split_once("@sha256:")
        .map(|(_, d)| format!("sha256:{d}"));
    let following_tag = transport == "oci" && pinned_digest.is_none();
    let owns = OwnsView::default();
    let badges = source_badges(&BadgeInputs {
        name: &source.name,
        pinned_digest: pinned_digest.as_deref(),
        active_digest: None,
        following_tag,
        restart_pending: false,
        kit_access: owns
            .marketplaces
            .iter()
            .filter(|m| declaring_access.contains(m))
            .map(String::as_str)
            .collect(),
    });
    BundleSourceView {
        name: source.name.clone(),
        owner_key: format!("bundle:{}", source.name),
        transport,
        reference,
        pinned_digest,
        following_tag,
        active_digest: None,
        version: None,
        content_hash: None,
        fetched_at: None,
        owns,
        badges,
        pin_command: format!(
            "just services-pin {} <digest>   # rewrites services.sources[{}] in the profile",
            source.name, source.name
        ),
    }
}

// Why: a kit that ships its own `access:` is a second truth for who reaches
// it. Core still ingests it at reconcile (source `bundle:<name>`), so the
// panel names it and the access-control drift shows those rows as bundle
// orphans that an overwrite deletes.
fn marketplaces_declaring_access(env: &AdminEnv) -> Vec<String> {
    env.services()
        .marketplaces
        .iter()
        .filter(|(_, m)| m.access.declares_rules() || m.access.default_included)
        .map(|(id, _)| id.as_str().to_owned())
        .collect()
}

fn refresh_hint(env: &AdminEnv) -> String {
    format!(
        "curl -X POST -H \"Authorization: Bearer $ADMIN_TOKEN\" \
         http://127.0.0.1:{}/api/v1/admin/services/refresh",
        env.config().port
    )
}
