//! A marketplace version is its content hash: stable under unrelated
//! changes, moved by a single byte in a skill it ships, identical for a base
//! and a bundled marketplace with the same bytes, and carrying a manifest
//! whose digests let two versions diff without re-reading either tree.

use systemprompt::identifiers::{MarketplaceId, PluginId, SkillId};
use systemprompt::models::services::bundle::FileEntry;
use systemprompt_web_admin::repositories::sync::marketplace_hash::{
    MarketplaceSpec, MarketplaceVersion, PluginSpec, SourceOf, hash_marketplaces, skill_key,
};

fn file(path: &str, content: &str) -> FileEntry {
    FileEntry {
        path: path.to_owned(),
        sha256: format!("{:x}", md5_like(content)),
        size: content.len() as u64,
    }
}

// A cheap, deterministic stand-in for a digest: the tests only need that
// different content yields a different string.
fn md5_like(content: &str) -> u64 {
    content.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn tree() -> Vec<FileEntry> {
    vec![
        file(
            "marketplaces/india/config.yaml",
            "id: india\nplugins: [ba]\n",
        ),
        file(
            "marketplaces/commons/config.yaml",
            "id: commons\nplugins: [ba]\n",
        ),
        file(
            "plugins/ba/config.yaml",
            "id: ba\nskills: [jira_management]\n",
        ),
        file(
            "skills/jira_management/config.yaml",
            "id: jira_management\n",
        ),
        file("skills/jira_management/SKILL.md", "# Jira\n"),
        file("skills/unrelated/SKILL.md", "# Not in any plugin\n"),
        file("rules/india.md", "irrelevant to any marketplace"),
    ]
}

fn specs() -> Vec<MarketplaceSpec> {
    let ba = PluginSpec {
        id: PluginId::new("ba"),
        skills: vec![SkillId::new("jira_management")],
    };
    vec![
        MarketplaceSpec {
            id: MarketplaceId::new("commons"),
            name: "Commons".into(),
            version: "1.0.0".into(),
            plugins: vec![ba.clone()],
        },
        MarketplaceSpec {
            id: MarketplaceId::new("india"),
            name: "India".into(),
            version: "2.0.0".into(),
            plugins: vec![ba],
        },
    ]
}

fn base(_: &MarketplaceId) -> SourceOf {
    SourceOf {
        source: "base".into(),
        source_hash: Some("treehash".into()),
    }
}

#[test]
fn the_hash_moves_only_with_the_bytes_the_marketplace_delivers() {
    let before = hash_marketplaces(&tree(), &specs(), base);
    let india = |v: &[MarketplaceVersion]| {
        v.iter()
            .find(|m| m.marketplace_id.as_str() == "india")
            .map(|m| m.content_hash.clone())
            .unwrap_or_default()
    };
    let mut unrelated = tree();
    unrelated[5] = file(
        "skills/unrelated/SKILL.md",
        "# Changed, but nobody ships it\n",
    );
    unrelated[6] = file("rules/india.md", "rules are not part of a marketplace");
    assert_eq!(
        india(&before),
        india(&hash_marketplaces(&unrelated, &specs(), base))
    );

    let mut moved = tree();
    moved[4] = file(
        "skills/jira_management/SKILL.md",
        "# Jira, one byte later\n",
    );
    let after = hash_marketplaces(&moved, &specs(), base);
    assert_ne!(
        india(&before),
        india(&after),
        "a skill byte moves the version"
    );
    assert_eq!(
        before[0].manifest.plugins[0].skills[0].skill_id.as_str(),
        "jira_management"
    );
    assert_ne!(
        before[0].manifest.plugins[0].skills[0].digest,
        after[0].manifest.plugins[0].skills[0].digest,
        "the manifest's per-skill digest moves with it, so the diff can name the skill"
    );
    assert_eq!(
        before[0].manifest.plugins[0].digest, after[0].manifest.plugins[0].digest,
        "the plugin's own files did not move"
    );
}

#[test]
fn two_marketplaces_sharing_bytes_hash_differently_only_through_their_own_config() {
    let versions = hash_marketplaces(&tree(), &specs(), base);
    assert_eq!(versions.len(), 2);
    assert_ne!(versions[0].content_hash, versions[1].content_hash);
    assert_eq!(versions[0].plugin_count, 1);
    assert_eq!(versions[0].skill_count, 1);
    assert_eq!(
        versions[0].manifest.files, 4,
        "config + plugin + two skill files"
    );
    assert_eq!(versions[0].source, "base");
    assert_eq!(versions[0].source_hash.as_deref(), Some("treehash"));
}

#[test]
fn a_bundled_marketplace_carries_the_same_kind_of_hash_with_its_bundle_as_provenance() {
    let bundled = |id: &MarketplaceId| SourceOf {
        source: if id.as_str() == "india" {
            "bundle:acme-ba".into()
        } else {
            "base".into()
        },
        source_hash: Some(if id.as_str() == "india" {
            "kithash".into()
        } else {
            "treehash".into()
        }),
    };
    let as_base = hash_marketplaces(&tree(), &specs(), base);
    let as_bundle = hash_marketplaces(&tree(), &specs(), bundled);
    assert_eq!(
        as_base[1].content_hash, as_bundle[1].content_hash,
        "identity is the bytes, never the source that shipped them"
    );
    assert_eq!(as_bundle[1].source, "bundle:acme-ba");
    assert_eq!(as_bundle[1].source_hash.as_deref(), Some("kithash"));
}

#[test]
fn a_missing_skill_directory_still_shapes_the_version() {
    let mut specs = specs();
    specs[1].plugins[0]
        .skills
        .push(SkillId::new("does_not_exist"));
    let versions = hash_marketplaces(&tree(), &specs, base);
    let india = &versions[1];
    assert_eq!(india.skill_count, 2);
    let missing = &india.manifest.plugins[0].skills[0];
    assert_eq!(missing.skill_id.as_str(), "does_not_exist");
    assert!(
        !missing.digest.is_empty(),
        "a broken reference hashes as an empty entry, not nothing"
    );
}

#[test]
fn skill_keys_take_the_dashed_form_invocation_facts_carry() {
    assert_eq!(
        skill_key("acme-ba", "jira_management"),
        "acme-ba:jira-management"
    );
    let versions = hash_marketplaces(&tree(), &specs(), base);
    assert_eq!(
        versions[0].manifest.plugins[0].skills[0].skill_key,
        "ba:jira-management"
    );
}
