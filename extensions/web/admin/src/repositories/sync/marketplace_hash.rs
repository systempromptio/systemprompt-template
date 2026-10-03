//! A marketplace's content hash — the identity of a marketplace version.
//!
//! sha256 over the marketplace's own directory, every plugin it includes and
//! every skill those plugins ship, computed with the same file walk and
//! digest a bundle's `content_hash` and the base `tree_hash` use. A base
//! marketplace and a bundled one therefore carry the same kind of identity,
//! and the bundle or tree hash stays beside it as provenance. The manifest
//! keeps a digest per plugin and per skill so two versions diff without
//! re-reading either tree.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{MarketplaceId, PluginId, SkillId};
use systemprompt::loader::bundle::pack::collect_files;
use systemprompt::manifest::services::ServicesConfig;
use systemprompt::manifest::services::bundle::{FileEntry, ServicesBundleManifest};

use super::sources::SourcesView;
use super::sources_db::BASE_SOURCE;
use super::tree_hash::{Fingerprint, tree_fingerprint};

const VERSION_DIRS: &[&str] = &["marketplaces", "plugins", "skills"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillManifest {
    pub skill_id: SkillId,
    pub skill_key: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin_id: PluginId,
    pub digest: String,
    pub skills: Vec<SkillManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    pub marketplace_id: MarketplaceId,
    pub name: String,
    pub version: String,
    pub plugins: Vec<PluginManifest>,
    pub files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarketplaceVersion {
    pub marketplace_id: MarketplaceId,
    pub content_hash: String,
    pub source: String,
    pub source_hash: Option<String>,
    pub manifest: MarketplaceManifest,
    pub plugin_count: i32,
    pub skill_count: i32,
}

/// A marketplace as the hash sees it: its id, the plugins it includes and
/// the skills each plugin ships, read off the composed config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketplaceSpec {
    pub id: MarketplaceId,
    pub name: String,
    pub version: String,
    pub plugins: Vec<PluginSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSpec {
    pub id: PluginId,
    pub skills: Vec<SkillId>,
}

/// Where a marketplace's declarations come from and that source's hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOf {
    pub source: String,
    pub source_hash: Option<String>,
}

type Cached = Option<(Fingerprint, Vec<MarketplaceVersion>)>;

// Why: hashing reads every file under three directories, so the result is
// cached per process and recomputed only when those directories move.
pub fn compute_marketplace_versions(
    root: &Path,
    services: &ServicesConfig,
    sources: &SourcesView,
) -> Vec<MarketplaceVersion> {
    static CACHE: OnceLock<Mutex<Cached>> = OnceLock::new();
    let Some(fingerprint) = tree_fingerprint(root, VERSION_DIRS) else {
        return Vec::new();
    };
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock()
        && let Some((fp, versions)) = guard.as_ref()
        && *fp == fingerprint
    {
        return versions.clone();
    }
    let Ok(files) = collect_files(root, VERSION_DIRS) else {
        return Vec::new();
    };
    let owner = source_lookup(sources);
    let versions = hash_marketplaces(&files, &specs_of(services), |id| owner(id.as_str()));
    if let Ok(mut guard) = cache.lock() {
        *guard = Some((fingerprint, versions.clone()));
    }
    versions
}

pub fn specs_of(services: &ServicesConfig) -> Vec<MarketplaceSpec> {
    let mut marketplaces: Vec<_> = services.marketplaces.iter().collect();
    marketplaces.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
    marketplaces
        .into_iter()
        .map(|(id, m)| MarketplaceSpec {
            id: id.clone(),
            name: m.name.clone(),
            version: m.version.clone(),
            plugins: m
                .plugins
                .include
                .iter()
                .map(|plugin_id| PluginSpec {
                    id: PluginId::new(plugin_id.clone()),
                    skills: services
                        .plugins
                        .values()
                        .find(|p| p.id.as_str() == plugin_id)
                        .map(|p| p.skills.include.iter().cloned().map(SkillId::new).collect())
                        .unwrap_or_default(),
                })
                .collect(),
        })
        .collect()
}

pub fn hash_marketplaces(
    files: &[FileEntry],
    specs: &[MarketplaceSpec],
    owner: impl Fn(&MarketplaceId) -> SourceOf,
) -> Vec<MarketplaceVersion> {
    specs
        .iter()
        .map(|spec| hash_one(files, spec, owner(&spec.id)))
        .collect()
}

fn hash_one(files: &[FileEntry], spec: &MarketplaceSpec, owner: SourceOf) -> MarketplaceVersion {
    let id = spec.id.as_str();
    let mut selected: Vec<FileEntry> = under(files, &format!("marketplaces/{id}/"));
    let mut plugins: Vec<&PluginSpec> = spec.plugins.iter().collect();
    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    plugins.dedup_by(|a, b| a.id == b.id);
    let plugins: Vec<PluginManifest> = plugins
        .into_iter()
        .map(|plugin| {
            let plugin_files = under(files, &format!("plugins/{}/", plugin.id));
            let mut skill_ids: Vec<&str> = plugin.skills.iter().map(SkillId::as_str).collect();
            skill_ids.sort_unstable();
            skill_ids.dedup();
            let skills = skill_ids
                .into_iter()
                .map(|skill_id| {
                    let skill_files = under(files, &format!("skills/{skill_id}/"));
                    let manifest = SkillManifest {
                        skill_id: SkillId::new(skill_id),
                        skill_key: skill_key(plugin.id.as_str(), skill_id),
                        digest: digest_of(&skill_files, &format!("skills/{skill_id}")),
                    };
                    selected.extend(skill_files);
                    manifest
                })
                .collect();
            let manifest = PluginManifest {
                plugin_id: plugin.id.clone(),
                digest: digest_of(&plugin_files, &format!("plugins/{}", plugin.id)),
                skills,
            };
            selected.extend(plugin_files);
            manifest
        })
        .collect();
    // Why: a skill two plugins share is one set of bytes; hashing it twice
    // would make the hash depend on plugin count, not content.
    selected.sort_by(|a, b| a.path.cmp(&b.path));
    selected.dedup_by(|a, b| a.path == b.path);
    let skill_count = plugins.iter().map(|p| p.skills.len()).sum::<usize>();
    MarketplaceVersion {
        marketplace_id: spec.id.clone(),
        content_hash: ServicesBundleManifest::compute_content_hash(&selected),
        source: owner.source,
        source_hash: owner.source_hash,
        plugin_count: i32::try_from(plugins.len()).unwrap_or(i32::MAX),
        skill_count: i32::try_from(skill_count).unwrap_or(i32::MAX),
        manifest: MarketplaceManifest {
            marketplace_id: spec.id.clone(),
            name: spec.name.clone(),
            version: spec.version.clone(),
            files: selected.len(),
            plugins,
        },
    }
}

fn under(files: &[FileEntry], prefix: &str) -> Vec<FileEntry> {
    files
        .iter()
        .filter(|f| f.path.starts_with(prefix))
        .cloned()
        .collect()
}

// Why: an included id whose directory is missing still shapes the version —
// the reference is broken — so it hashes as a single empty entry rather
// than vanishing.
fn digest_of(files: &[FileEntry], missing_path: &str) -> String {
    if files.is_empty() {
        return ServicesBundleManifest::compute_content_hash(&[FileEntry {
            path: missing_path.to_owned(),
            sha256: String::new(),
            size: 0,
        }]);
    }
    ServicesBundleManifest::compute_content_hash(files)
}

pub fn skill_key(plugin: &str, skill: &str) -> String {
    crate::util::skill_ref::skill_ref(plugin, skill)
}

fn source_lookup(sources: &SourcesView) -> impl Fn(&str) -> SourceOf + '_ {
    let mut by_marketplace: BTreeMap<&str, (&str, Option<&str>)> = BTreeMap::new();
    for b in &sources.bundles {
        for m in &b.owns.marketplaces {
            by_marketplace.insert(
                m.as_str(),
                (b.owner_key.as_str(), b.content_hash.as_deref()),
            );
        }
    }
    move |id: &str| {
        by_marketplace.get(id).map_or_else(
            || SourceOf {
                source: BASE_SOURCE.to_owned(),
                source_hash: sources.base.tree_hash.clone(),
            },
            |(owner, hash)| SourceOf {
                source: (*owner).to_owned(),
                source_hash: hash.map(str::to_owned),
            },
        )
    }
}
