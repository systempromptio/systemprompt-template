//! The inline diff between two marketplace versions is derived from their
//! stored manifests alone: added, removed and changed skills by digest, and
//! plugins likewise; a first version diffs against nothing.

use systemprompt::identifiers::{MarketplaceId, PluginId, SkillId};
use systemprompt_web_admin::repositories::sync::marketplace_hash::{
    MarketplaceManifest, PluginManifest, SkillManifest,
};
use systemprompt_web_admin::test_support::{Change, manifest_diff};

fn skill(plugin: &str, id: &str, digest: &str) -> SkillManifest {
    SkillManifest {
        skill_id: SkillId::new(id),
        skill_key: format!("{plugin}:{}", id.replace('_', "-")),
        digest: digest.to_owned(),
    }
}

fn manifest(plugins: Vec<(&str, &str, Vec<SkillManifest>)>) -> MarketplaceManifest {
    MarketplaceManifest {
        marketplace_id: MarketplaceId::new("india"),
        name: "India".into(),
        version: "1".into(),
        files: 0,
        plugins: plugins
            .into_iter()
            .map(|(id, digest, skills)| PluginManifest {
                plugin_id: PluginId::new(id),
                digest: digest.into(),
                skills,
            })
            .collect(),
    }
}

#[test]
fn skills_are_labelled_added_removed_changed_or_unchanged_by_digest() {
    let before = manifest(vec![(
        "ba",
        "p1",
        vec![
            skill("ba", "jira_management", "s1"),
            skill("ba", "bug_logging", "s2"),
            skill("ba", "old", "s9"),
        ],
    )]);
    let after = manifest(vec![(
        "ba",
        "p1",
        vec![
            skill("ba", "jira_management", "s1-changed"),
            skill("ba", "bug_logging", "s2"),
            skill("ba", "brand_new", "s3"),
        ],
    )]);
    let d = manifest_diff(Some(&before), &after);
    assert_eq!((d.added, d.removed, d.changed, d.unchanged), (1, 1, 1, 1));
    let by_id = |id: &str| {
        d.skills
            .iter()
            .find(|s| s.skill_id.as_str() == id)
            .map(|s| s.change)
    };
    assert_eq!(by_id("brand_new"), Some(Change::Added));
    assert_eq!(by_id("old"), Some(Change::Removed));
    assert_eq!(by_id("jira_management"), Some(Change::Changed));
    assert_eq!(by_id("bug_logging"), Some(Change::Unchanged));
    let changed = d
        .skills
        .iter()
        .find(|s| s.skill_id.as_str() == "jira_management")
        .expect("row");
    assert_eq!(changed.before.as_deref(), Some("s1"));
    assert_eq!(changed.after.as_deref(), Some("s1-changed"));
    assert_eq!(changed.skill_key, "ba:jira-management");
    assert!(d.has_changes());
    assert_eq!(d.moved_skills().len(), 3);
    assert_eq!(d.plugins.len(), 1);
    assert_eq!(
        d.plugins[0].change,
        Change::Unchanged,
        "a plugin whose own files did not move is unchanged even when its skills did"
    );
}

#[test]
fn plugins_added_or_removed_are_named_and_a_first_version_adds_everything() {
    let after = manifest(vec![
        ("ba", "p1", vec![skill("ba", "jira_management", "s1")]),
        ("sf", "p2", vec![skill("sf", "core_logic", "s5")]),
    ]);
    let first = manifest_diff(None, &after);
    assert_eq!(first.added, 2);
    assert!(first.plugins.iter().all(|p| p.change == Change::Added));

    let before = manifest(vec![(
        "ba",
        "p1",
        vec![skill("ba", "jira_management", "s1")],
    )]);
    let d = manifest_diff(Some(&before), &after);
    assert_eq!(
        d.plugins
            .iter()
            .find(|p| p.plugin_id.as_str() == "sf")
            .map(|p| p.change),
        Some(Change::Added)
    );
    let gone = manifest_diff(Some(&after), &before);
    assert_eq!(
        gone.plugins
            .iter()
            .find(|p| p.plugin_id.as_str() == "sf")
            .map(|p| p.change),
        Some(Change::Removed)
    );
    assert_eq!(gone.removed, 1);
    assert_eq!(Change::Removed.tone(), "err");
    assert_eq!(Change::Added.label(), "added");
}

#[test]
fn identical_manifests_have_nothing_to_say() {
    let m = manifest(vec![(
        "ba",
        "p1",
        vec![skill("ba", "jira_management", "s1")],
    )]);
    let d = manifest_diff(Some(&m), &m);
    assert!(!d.has_changes());
    assert_eq!(d.unchanged, 1);
    assert!(d.moved_skills().is_empty());
}
