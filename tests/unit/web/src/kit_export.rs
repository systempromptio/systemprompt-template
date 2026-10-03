//! The kit exporter must preserve the configured marketplace while producing
//! a portable tree that carries neither this instance's access grants nor its
//! MCP connection configuration.

use std::path::Path;

use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::split_frontmatter;
use systemprompt_kit_export::{export_kit, round_trip};

const MARKETPLACE: &str = "enterprise-demo";

fn services_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../services")
        .canonicalize()
        .expect("services tree")
}

#[test]
fn shipped_marketplace_export_round_trips_without_instance_access_or_mcp_files() {
    let services_root = services_root();
    let services = ConfigLoader::load_from_path(&services_root.join("config/config.yaml"))
        .expect("load configured services");
    let declared_plugins: Vec<String> = services
        .marketplaces
        .iter()
        .find(|(id, _)| id.as_str() == MARKETPLACE)
        .map(|(_, m)| m.plugins.include.clone())
        .expect("the shipped marketplace is configured");
    let out = tempfile::tempdir().expect("kit output directory");

    let report = export_kit(&services, &services_root, MARKETPLACE, out.path())
        .expect("export the shipped marketplace");
    assert_eq!(report.marketplace, MARKETPLACE);
    assert_eq!(report.plugins, declared_plugins);
    assert!(report.files > 0);
    let diffs = round_trip(out.path(), &services_root, &report).expect("strict re-import");
    assert!(
        diffs.is_empty(),
        "the kit preserves every portable marketplace, plugin, and skill declaration: {diffs:#?}"
    );

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(out.path().join(".claude-plugin/marketplace.json"))
            .expect("marketplace manifest"),
    )
    .expect("portable marketplace JSON");
    assert_eq!(manifest["name"], MARKETPLACE);
    let sidecar = std::fs::read_to_string(out.path().join(".claude-plugin/systemprompt.yaml"))
        .expect("marketplace sidecar");
    assert!(
        !sidecar.contains("access:"),
        "audience grants belong to the source instance, never the kit"
    );

    for plugin in &report.plugins {
        let dir = out.path().join("plugins").join(plugin);
        assert!(
            !dir.join(".mcp.json").exists(),
            "{plugin}: instance MCP connection configuration is not portable"
        );
        assert!(
            !dir.join("config.yaml").exists(),
            "{plugin}: the source config file, which can carry instance-only fields, is not copied"
        );
    }
    for skill in &report.skills {
        let kebab = skill.replace('_', "-");
        let exported = report
            .plugins
            .iter()
            .map(|p| {
                out.path()
                    .join("plugins")
                    .join(p)
                    .join("skills")
                    .join(&kebab)
                    .join("SKILL.md")
            })
            .find(|path| path.is_file())
            .unwrap_or_else(|| panic!("{skill}: no exported SKILL.md"));
        let exported = std::fs::read_to_string(exported).expect("exported skill instructions");
        let source =
            std::fs::read_to_string(services_root.join("skills").join(skill).join("SKILL.md"))
                .expect("source skill instructions");
        let body = split_frontmatter(&source)
            .expect("skill frontmatter and body")
            .body
            .trim();
        assert!(
            exported.contains(body),
            "{skill}: instruction body survives the frontmatter projection"
        );
    }
}

#[test]
fn export_refuses_an_unknown_marketplace_without_creating_a_tree() {
    let services_root = services_root();
    let services = ConfigLoader::load_from_path(&services_root.join("config/config.yaml"))
        .expect("load configured services");
    let out = tempfile::tempdir().expect("kit output directory");

    let error = export_kit(
        &services,
        &services_root,
        "missing-contract-marketplace",
        out.path(),
    )
    .expect_err("unknown marketplace must not export");
    assert!(
        error
            .to_string()
            .contains("no marketplace 'missing-contract-marketplace'")
    );
    assert!(
        std::fs::read_dir(out.path())
            .expect("empty output directory")
            .next()
            .is_none(),
        "failed lookup writes no partial kit"
    );
}
