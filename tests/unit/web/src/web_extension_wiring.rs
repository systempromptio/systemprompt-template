//! `WebExtension` is the single seam through which the five web sibling crates
//! reach the host runtime. Everything it advertises is consumed by name: the
//! `web` id keys its config prefix, `dependencies` decides migration order, and
//! `required_assets` is what `just publish` copies. The provider getters are
//! inventory-backed, so a missing `submit_*!` shows up as a provider that never
//! runs — asserted here as id uniqueness and a non-empty registry.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use systemprompt::extension::prelude::{AssetPaths, Extension};
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

#[test]
fn the_extension_identifies_itself_as_web_at_a_stable_priority() {
    let extension = WebExtension::new();
    let metadata = extension.metadata();

    assert_eq!(metadata.id, "web");
    assert!(!metadata.name.is_empty());
    assert!(!metadata.version.is_empty());
    assert_eq!(extension.priority(), 100);
    assert_eq!(extension.config_prefix(), Some(WebExtension::PREFIX));
    assert_eq!(WebExtension::PREFIX, "web");
}

#[test]
fn it_declares_the_extensions_whose_tables_it_reads() {
    let extension = WebExtension::new();

    let mut dependencies = extension.dependencies();
    dependencies.sort_unstable();
    assert_eq!(dependencies, vec!["authz", "content", "users"]);

    let mut shared = extension.cross_extension_tables();
    shared.sort_unstable();
    assert_eq!(
        shared,
        vec![
            "eval_cases",
            "eval_judge_calls",
            "eval_pairs",
            "eval_results",
            "eval_rubrics",
            "eval_runs",
            "markdown_content",
            "users",
        ]
    );
}

#[test]
fn required_assets_is_the_site_manifest_plus_the_admin_manifest() {
    let extension = WebExtension::new();
    assert!(extension.declares_assets());

    let paths = fake_paths();
    let required = extension.required_assets(&paths);
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
    let auth = WebExtension::new()
        .site_auth()
        .expect("the web extension gates its admin surface");

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
