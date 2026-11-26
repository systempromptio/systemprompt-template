//! `WebExtension` is the single seam through which the five web sibling crates
//! reach the host runtime. Everything it contributes is consumed by name: the
//! `web` id keys its config, the cross-extension tables decide migration
//! order, and the contributed assets are what `just publish` copies.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use systemprompt::extension::contribute::ContributionSink;
use systemprompt::extension::prelude::AssetPaths;
use systemprompt::extension::{Admitted, Extension};
use systemprompt_web_extension::WebExtension;
use systemprompt_web_site::web_assets;

struct FakePaths {
    storage: PathBuf,
    dist: PathBuf,
}

impl AssetPaths for FakePaths {
    fn storage_files(&self) -> &Path {
        &self.storage
    }

    fn web_dist(&self) -> &Path {
        &self.dist
    }
}

fn fake_paths() -> FakePaths {
    FakePaths {
        storage: PathBuf::from("/srv/storage/files"),
        dist: PathBuf::from("/srv/web/dist"),
    }
}

fn contributed() -> Admitted {
    template_test_common::composition::admitted()
}

fn web(admitted: &Admitted) -> &ContributionSink {
    &template_test_common::composition::member(admitted, WebExtension::ID).sink
}

#[test]
fn the_extension_identifies_itself_as_web() {
    let metadata = WebExtension::new().metadata();

    assert_eq!(metadata.id, "web");
    assert_eq!(WebExtension::ID, "web");
    assert!(!metadata.name.is_empty());
    assert!(!metadata.version.is_empty());
}

#[test]
fn it_declares_the_tables_it_reads_from_other_extensions() {
    let admitted = contributed();
    let sink = web(&admitted);
    let mut shared = sink.cross_tables().to_vec();
    shared.sort_unstable();
    assert_eq!(
        shared,
        vec![
            "managed_installation_receipts",
            "managed_publications",
            "managed_resources",
            "markdown_content",
            "mcp_tool_executions",
            "users",
        ]
    );
}

#[test]
fn contributed_assets_are_the_site_manifest_plus_the_admin_manifest() {
    let admitted = contributed();
    let sink = web(&admitted);
    assert!(!sink.assets().is_empty());

    let paths = fake_paths();
    let required: Vec<_> = sink.assets().iter().flat_map(|f| f(&paths)).collect();
    let site = web_assets(&paths);

    assert!(
        required.len() > site.len(),
        "required assets must add the admin manifest on top of the site one"
    );

    let declared: HashSet<&str> = required.iter().map(|a| a.destination()).collect();
    for asset in &site {
        assert!(
            declared.contains(asset.destination()),
            "{} is declared by the site but not by the extension",
            asset.destination()
        );
    }
    assert_eq!(
        declared.len(),
        required.len(),
        "two assets publish to the same destination"
    );
}

#[test]
fn the_admin_surface_is_login_gated_and_leaves_the_public_site_open() {
    let admitted = contributed();
    let sink = web(&admitted);
    let [auth] = sink.site_auth() else {
        panic!("the web extension gates its admin surface exactly once");
    };

    assert_eq!(auth.login_path, "/admin/login");
    assert_eq!(auth.required_scope, "user");
    assert!(auth.protected_prefixes.contains(&"/admin"));
    for public in auth.public_prefixes {
        assert!(
            public.starts_with("/admin"),
            "{public} is not under a protected prefix, so exempting it is dead config"
        );
    }
    assert!(
        auth.public_prefixes.contains(&"/admin/login"),
        "the login page itself must not require a login"
    );
}
