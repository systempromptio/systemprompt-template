//! The subject dimensions this extension adds, and the pure shape-shuffling
//! the marketplace filter does around the access-control resolver.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use std::collections::{BTreeMap, BTreeSet};

use systemprompt::identifiers::{AgentId, HookId, MarketplaceId, McpServerId};
use systemprompt::marketplace::{EntryKeepSets, MarketplaceCandidate, MarketplaceMembership};
use systemprompt::models::bridge::ids::{LibraryArtifactId, PluginId, SkillId};
use systemprompt::models::bridge::manifest::{
    AgentEntry, ArtifactEntry, HookEntry, ManagedMcpServer, PluginEntry, SkillEntry,
};
use systemprompt::models::services::MarketplaceAccess;
use systemprompt_security::authz::{EntityKind, EntityRef};
use systemprompt_web_admin::authz::group::{group_dimension, group_rule_type};
use systemprompt_web_admin::authz::project::{project_dimension, project_rule_type};
use systemprompt_web_admin::authz::salesforce::salesforce_dimension;

#[test]
fn the_membership_dimensions_sit_between_user_and_role() {
    let group = group_dimension();
    assert_eq!(group.precedence, 150);
    assert_eq!(group.rule_type, group_rule_type());
    assert_eq!(group_rule_type().as_str(), "group");

    let project = project_dimension();
    assert_eq!(project.precedence, 140);
    assert_eq!(project.rule_type, project_rule_type());
    assert_eq!(project_rule_type().as_str(), "project");
}

#[test]
fn the_extension_dimensions_are_distinct_and_ordered() {
    let project = project_dimension();
    let group = group_dimension();
    let salesforce = salesforce_dimension();
    assert_ne!(project.rule_type, group.rule_type);
    assert_ne!(group.rule_type, salesforce.rule_type);
    assert!(
        project.precedence < group.precedence && group.precedence < salesforce.precedence,
        "a project is the narrowest statement, then group, then the linked-account band"
    );
    assert!(salesforce.precedence < 200, "and all sit below role");
}

#[test]
fn every_entity_kind_maps_to_its_own_typed_reference() {
    let mut seen = Vec::new();
    for kind in EntityKind::ALL {
        let entity = EntityRef::from_kind_and_id(*kind, "id-1");
        assert_eq!(entity.kind(), *kind);
        assert_eq!(entity.id_str(), "id-1");
        assert!(
            !seen.contains(&std::mem::discriminant(&entity)),
            "{kind:?} reuses another kind's EntityRef variant"
        );
        seen.push(std::mem::discriminant(&entity));
    }
    assert!(matches!(
        EntityRef::from_kind_and_id(EntityKind::Plugin, "p"),
        EntityRef::Plugin(_)
    ));
}

fn owned_by(artifact: &str, plugin: &str) -> (LibraryArtifactId, BTreeSet<PluginId>) {
    (
        LibraryArtifactId::try_new(artifact).expect("artifact id"),
        BTreeSet::from([PluginId::try_new(plugin).expect("plugin id")]),
    )
}

fn candidate() -> MarketplaceCandidate {
    MarketplaceCandidate {
        plugins: vec![
            plugin_entry("astound-admin"),
            plugin_entry("astound-commons"),
        ],
        skills: vec![skill_entry("skill-a"), skill_entry("skill-b")],
        agents: vec![agent_entry("agent-a")],
        hooks: vec![hook_entry("hook-a")],
        managed_mcp_servers: vec![mcp_entry("crm"), mcp_entry("systemprompt")],
        artifacts: vec![artifact_entry("art-admin"), artifact_entry("art-common")],
        ..MarketplaceCandidate::default()
    }
    .with_artifact_owners(BTreeMap::from([
        owned_by("art-admin", "astound-admin"),
        owned_by("art-common", "astound-commons"),
    ]))
}

fn plugin_entry(id: &str) -> PluginEntry {
    serde_json::from_value(serde_json::json!({
        "id": id, "version": "1.0.0", "sha256": "0".repeat(64), "files": [],
    }))
    .expect("plugin entry")
}

fn skill_entry(id: &str) -> SkillEntry {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "description": "", "file_path": "s.md",
        "sha256": "0".repeat(64), "instructions": "",
    }))
    .expect("skill entry")
}

fn agent_entry(id: &str) -> AgentEntry {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "display_name": id, "description": "", "version": "1.0.0",
        "endpoint": "", "enabled": true, "is_default": false, "is_primary": false,
    }))
    .expect("agent entry")
}

fn hook_entry(id: &str) -> HookEntry {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "description": "", "version": "1.0.0",
        "event": "PreToolUse", "matcher": "*", "command": "true",
        "category": "custom", "sha256": "0".repeat(64),
    }))
    .expect("hook entry")
}

fn mcp_entry(name: &str) -> ManagedMcpServer {
    serde_json::from_value(serde_json::json!({
        "name": name, "url": "https://example.test/mcp",
    }))
    .expect("mcp entry")
}

fn artifact_entry(id: &str) -> ArtifactEntry {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "description": "", "version": "1.0.0",
        "mcp_tools": [], "content": "<p></p>", "starred": false, "sha256": "0".repeat(64),
    }))
    .expect("artifact entry")
}

fn keep<T, F>(ids: &[&str], make: F) -> std::collections::HashSet<T>
where
    T: Eq + std::hash::Hash,
    F: Fn(&str) -> T,
{
    ids.iter().map(|s| make(s)).collect()
}

fn retained(mut input: MarketplaceCandidate, keep_sets: &EntryKeepSets) -> MarketplaceCandidate {
    input.retain_entries(keep_sets);
    input
}

#[test]
fn keep_sets_shrink_every_list_to_what_survived() {
    let kept = retained(
        candidate(),
        &EntryKeepSets {
            plugins: keep(&["astound-commons"], |s| {
                PluginId::try_new(s).expect("plugin id")
            }),
            skills: keep(&["skill-b"], |s| SkillId::try_new(s).expect("skill id")),
            agents: std::collections::HashSet::new(),
            hooks: keep(&["hook-a"], |s| HookId::new(s)),
            mcp_servers: keep(&["crm"], |s| McpServerId::new(s)),
            marketplaces: std::collections::HashSet::new(),
        },
    );
    assert_eq!(kept.plugins.len(), 1);
    assert_eq!(kept.plugins[0].id.as_str(), "astound-commons");
    assert_eq!(kept.skills.len(), 1);
    assert_eq!(kept.skills[0].id.as_str(), "skill-b");
    assert!(kept.agents.is_empty());
    assert_eq!(kept.hooks.len(), 1);
    assert_eq!(kept.managed_mcp_servers.len(), 1);
    assert_eq!(kept.managed_mcp_servers[0].name.as_str(), "crm");
}

#[test]
fn an_artifact_survives_only_while_one_of_its_owning_plugins_does() {
    let kept = retained(
        candidate(),
        &EntryKeepSets {
            plugins: keep(&["astound-commons"], |s| {
                PluginId::try_new(s).expect("plugin id")
            }),
            skills: std::collections::HashSet::new(),
            agents: std::collections::HashSet::new(),
            hooks: std::collections::HashSet::new(),
            mcp_servers: std::collections::HashSet::new(),
            marketplaces: std::collections::HashSet::new(),
        },
    );
    let ids: Vec<_> = kept.artifacts.iter().map(|a| a.id.to_string()).collect();
    assert_eq!(
        ids,
        ["art-common"],
        "the admin dashboard must not be staged"
    );
}

#[test]
fn dropping_every_plugin_drops_every_artifact() {
    let kept = retained(
        candidate(),
        &EntryKeepSets {
            plugins: std::collections::HashSet::new(),
            skills: keep(&["skill-a"], |s| SkillId::try_new(s).expect("skill id")),
            agents: std::collections::HashSet::new(),
            hooks: std::collections::HashSet::new(),
            mcp_servers: std::collections::HashSet::new(),
            marketplaces: std::collections::HashSet::new(),
        },
    );
    assert!(kept.artifacts.is_empty());
    assert_eq!(kept.skills.len(), 1, "other lists are unaffected");
}

#[test]
fn an_unowned_artifact_is_dropped_rather_than_defaulting_to_visible() {
    let mut input = candidate();
    input.artifact_owners.clear();
    let kept = retained(
        input,
        &EntryKeepSets {
            plugins: keep(&["astound-admin", "astound-commons"], |s| {
                PluginId::try_new(s).expect("plugin id")
            }),
            skills: std::collections::HashSet::new(),
            agents: std::collections::HashSet::new(),
            hooks: std::collections::HashSet::new(),
            mcp_servers: std::collections::HashSet::new(),
            marketplaces: std::collections::HashSet::new(),
        },
    );
    assert!(kept.artifacts.is_empty());
}

#[test]
fn the_assembly_context_passes_through_untouched() {
    let astound = MarketplaceId::new("astound");
    let membership = MarketplaceMembership {
        access: BTreeMap::from([(astound.clone(), MarketplaceAccess::default())]),
        ..MarketplaceMembership::default()
    };
    let mut input = candidate().with_membership(membership);
    input.diagnostics.push("assembly warning".to_owned());
    let owners = input.artifact_owners.clone();

    let kept = retained(
        input,
        &EntryKeepSets {
            plugins: keep(&["astound-admin"], |s| {
                PluginId::try_new(s).expect("plugin id")
            }),
            skills: std::collections::HashSet::new(),
            agents: std::collections::HashSet::new(),
            hooks: std::collections::HashSet::new(),
            mcp_servers: std::collections::HashSet::new(),
            marketplaces: std::collections::HashSet::new(),
        },
    );
    assert_eq!(kept.artifact_owners, owners);
    assert_eq!(
        kept.membership.all_ids(),
        BTreeSet::from([astound]),
        "membership is assembly context, not an entry list, so filtering leaves it alone"
    );
    assert_eq!(kept.diagnostics, vec!["assembly warning".to_owned()]);
}

#[test]
fn keeping_everything_is_the_identity() {
    let kept = retained(
        candidate(),
        &EntryKeepSets {
            plugins: keep(&["astound-admin", "astound-commons"], |s| {
                PluginId::try_new(s).expect("plugin id")
            }),
            skills: keep(&["skill-a", "skill-b"], |s| {
                SkillId::try_new(s).expect("skill id")
            }),
            agents: keep(&["agent-a"], |s| AgentId::new(s)),
            hooks: keep(&["hook-a"], |s| HookId::new(s)),
            mcp_servers: keep(&["crm", "systemprompt"], |s| McpServerId::new(s)),
            marketplaces: std::collections::HashSet::new(),
        },
    );
    assert_eq!(kept.plugins.len(), 2);
    assert_eq!(kept.skills.len(), 2);
    assert_eq!(kept.agents.len(), 1);
    assert_eq!(kept.hooks.len(), 1);
    assert_eq!(kept.managed_mcp_servers.len(), 2);
    assert_eq!(kept.artifacts.len(), 2);
}
