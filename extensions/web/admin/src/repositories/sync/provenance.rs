//! Which source a piece of configuration came from: the base tree this
//! repository ships, or a bundle the profile pins.
//!
//! Ownership is decided at composition and recorded in each bundle's
//! manifest as the files it carries; the base owns whatever no bundle
//! does. A directory kind can be both — `skills/` holds the base's skills
//! and a kit's — and the page says so rather than picking one.

use serde::Serialize;
use systemprompt::config::ProfileBootstrap;
use systemprompt::loader::bundle::{BundleCache, cache_root};

use super::sources::SourcesView;
use crate::error::AdminResult;

pub const BASE_SOURCE: &str = "base";

#[derive(Debug, Clone, Serialize)]
pub struct Provenance {
    pub source: String,
    pub tone: &'static str,
    pub mixed: bool,
}

impl Provenance {
    #[must_use]
    pub fn base() -> Self {
        Self {
            source: BASE_SOURCE.to_owned(),
            tone: "ok",
            mixed: false,
        }
    }

    fn bundle(name: &str) -> Self {
        Self {
            source: format!("bundle:{name}"),
            tone: "info",
            mixed: false,
        }
    }
}

/// The files each active bundle carries, read once per page.
#[derive(Debug, Clone, Default)]
pub struct BundleFiles {
    pub name: String,
    pub paths: Vec<String>,
}

pub fn active_bundle_files() -> AdminResult<Vec<BundleFiles>> {
    let profile = ProfileBootstrap::get()?;
    let cache = BundleCache::new(cache_root(profile));
    let state = cache.read_state();
    Ok(state
        .sources
        .iter()
        .filter_map(|(name, active)| {
            let manifest = cache.read_manifest(name, &active.content_hash).ok()?;
            Some(BundleFiles {
                name: name.clone(),
                paths: manifest
                    .manifest
                    .files
                    .into_iter()
                    .map(|f| f.path)
                    .collect(),
            })
        })
        .collect())
}

fn under(path: &str, rel: &str, is_dir: bool) -> bool {
    if is_dir {
        path.strip_prefix(rel)
            .is_some_and(|rest| rest.starts_with('/'))
    } else {
        path == rel
    }
}

// Why: `base_has` is whether the baked tree holds the path; a kind only a
// bundle ships is that bundle's, one both ship is mixed.
#[must_use]
pub fn source_for_path(
    bundles: &[BundleFiles],
    rel: &str,
    is_dir: bool,
    base_has: bool,
) -> Provenance {
    let owners: Vec<&str> = bundles
        .iter()
        .filter(|b| b.paths.iter().any(|p| under(p, rel, is_dir)))
        .map(|b| b.name.as_str())
        .collect();
    match (owners.as_slice(), base_has) {
        ([], _) => Provenance::base(),
        ([one], false) => Provenance::bundle(one),
        (many, base) => {
            let mut names: Vec<String> = many.iter().map(|n| format!("bundle:{n}")).collect();
            if base {
                names.insert(0, BASE_SOURCE.to_owned());
            }
            Provenance {
                source: names.join(" + "),
                tone: "info",
                mixed: true,
            }
        },
    }
}

/// What a catalog row is: a marketplace, plugin or skill id.
#[derive(Debug, Clone, Copy)]
pub enum OwnedKind {
    Marketplace,
    Plugin,
    Skill,
}

#[must_use]
pub fn owned_by(sources: &SourcesView, kind: OwnedKind, id: &str) -> Provenance {
    sources
        .bundles
        .iter()
        .find(|b| {
            let ids = match kind {
                OwnedKind::Marketplace => &b.owns.marketplaces,
                OwnedKind::Plugin => &b.owns.plugins,
                OwnedKind::Skill => &b.owns.skills,
            };
            ids.iter().any(|o| o == id)
        })
        .map_or_else(Provenance::base, |b| Provenance::bundle(&b.name))
}
