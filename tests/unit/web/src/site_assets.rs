//! The asset manifest is the contract between `storage/files/` and `web/dist/`:
//! `just publish` copies exactly what is declared here. Two things can silently
//! break a deploy — a source path assembled off the wrong storage root, and a
//! duplicate destination, where one file quietly overwrites another. Both are
//! pinned here, along with the deliberate omission of
//! `storage/files/downloads`, which is served straight from storage and must
//! not be copied.

use std::path::{Path, PathBuf};
use systemprompt::extension::{AssetPaths, AssetType};
use systemprompt_web_site::assets::css::css_assets;
use systemprompt_web_site::assets::js_services::{public_js_assets, service_js_assets};
use systemprompt_web_site::web_assets;

struct FakePaths {
    storage: PathBuf,
    dist: PathBuf,
}

impl FakePaths {
    fn new() -> Self {
        Self {
            storage: PathBuf::from("/srv/storage/files"),
            dist: PathBuf::from("/srv/web/dist"),
        }
    }
}

impl AssetPaths for FakePaths {
    fn storage_files(&self) -> &Path {
        &self.storage
    }

    fn web_dist(&self) -> &Path {
        &self.dist
    }
}

#[test]
fn css_sources_hang_off_the_storage_css_root_and_publish_under_css() {
    let assets = css_assets(Path::new("/srv/storage/files/css"));
    assert!(!assets.is_empty());

    for asset in &assets {
        assert_eq!(asset.asset_type(), AssetType::Css);
        assert!(
            asset.source().starts_with("/srv/storage/files/css"),
            "{} is not rooted in storage css",
            asset.source().display()
        );
        assert!(
            asset.destination().starts_with("css/"),
            "{} must publish under css/",
            asset.destination()
        );
    }
}

#[test]
fn javascript_sources_hang_off_the_storage_js_root_and_publish_under_js() {
    let root = Path::new("/srv/storage/files/js");
    let mut assets = public_js_assets(root);
    assets.extend(service_js_assets(root));

    for asset in &assets {
        assert_eq!(asset.asset_type(), AssetType::JavaScript);
        assert!(asset.source().starts_with(root));
        assert!(
            asset.destination().starts_with("js/"),
            "{} must publish under js/",
            asset.destination()
        );
    }
    assert!(
        service_js_assets(root)
            .iter()
            .any(|a| a.destination() == "js/services/webauthn-login.js")
    );
}

#[test]
fn the_full_manifest_has_unique_destinations_and_skips_client_downloads() {
    let assets = web_assets(&FakePaths::new());

    let mut destinations: Vec<&str> = assets.iter().map(|a| a.destination()).collect();
    destinations.sort_unstable();
    let count = destinations.len();
    destinations.dedup();
    assert_eq!(
        destinations.len(),
        count,
        "two assets publish to the same destination"
    );

    assert!(
        !assets
            .iter()
            .any(|a| a.destination().starts_with("downloads/")),
        "client downloads are served from storage and must not be copied to dist"
    );
}
