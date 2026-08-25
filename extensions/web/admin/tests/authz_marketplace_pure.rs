//! The subject dimensions this extension adds, and the pure shape-shuffling
//! the marketplace filter does around the access-control resolver.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test fixtures assert by panicking, exactly as the tests/ workspace allows"
)]

use std::collections::{BTreeMap, BTreeSet};

use systemprompt::identifiers::{AgentId, HookId, McpServerId};
use systemprompt::marketplace::{EntryKeepSets, MarketplaceCandidate};
use systemprompt::models::bridge::ids::{LibraryArtifactId, PluginId, SkillId};
use systemprompt::models::bridge::manifest::{
    AgentEntry, ArtifactEntry, HookEntry, ManagedMcpServer, PluginEntry, SkillEntry,
};
use systemprompt_security::authz::{EntityKind, EntityRef};
use systemprompt_web_admin::authz::department::{department_dimension, department_rule_type};

#[test]
fn the_department_dimension_sits_between_user_and_role() {
    let dimension = department_dimension();
    assert_eq!(dimension.label, "Department");
    assert_eq!(dimension.precedence, 100);
    assert_eq!(dimension.rule_type, department_rule_type());
    assert_eq!(department_rule_type().as_str(), "department");
}

#[test]
fn department_is_the_only_dimension_this_extension_adds() {
    let dimension = department_dimension();
    assert!(
        dimension.precedence > 0 && dimension.precedence < 200,
        "a department rule must refine a user rule and be refined by a role rule"
    );
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
        plugins: vec![plugin_entry("demo-admin"), plugin_entry("demo-commons")],
        skills: vec![skill_entry("skill-a"), skill_entry("skill-b")],
        agents: vec![agent_entry("agent-a")],
        hooks: vec![hook_entry("hook-a")],
        managed_mcp_servers: vec![mcp_entry("files"), mcp_entry("systemprompt")],
        artifacts: vec![artifact_entry("art-admin"), artifact_entry("art-common")],
        ..MarketplaceCandidate::default()
    }
    .with_artifact_owners(BTreeMap::from([
        owned_by("art-admin", "demo-admin"),
        owned_by("art-common", "demo-commons"),
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
            plugins: keep(&["demo-commons"], |s| PluginId::try_new(s).expect("plugin id")),
            skills: keep(&["skill-b"], |s| SkillId::try_new(s).expect("skill id")),
            agents: std::collections::HashSet::new(),
            hooks: keep(&["hook-a"], |s| HookId::new(s)),
            mcp_servers: keep(&["files"], |s| McpServerId::new(s)),
        },
    );
    assert_eq!(kept.plugins.len(), 1);
    assert_eq!(kept.plugins[0].id.as_str(), "demo-commons");
    assert_eq!(kept.skills.len(), 1);
    assert_eq!(kept.skills[0].id.as_str(), "skill-b");
    assert!(kept.agents.is_empty());
    assert_eq!(kept.hooks.len(), 1);
    assert_eq!(kept.managed_mcp_servers.len(), 1);
    assert_eq!(kept.managed_mcp_servers[0].name.as_str(), "files");
}

#[test]
fn an_artifact_survives_only_while_one_of_its_owning_plugins_does() {
    let kept = retained(
        candidate(),
        &EntryKeepSets {
            plugins: keep(&["demo-commons"], |s| PluginId::try_new(s).expect("plugin id")),
            skills: std::collections::HashSet::new(),
            agents: std::collections::HashSet::new(),
            hooks: std::collections::HashSet::new(),
            mcp_servers: std::collections::HashSet::new(),
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
            plugins: keep(&["demo-admin", "demo-commons"], |s| PluginId::try_new(s).expect("plugin id")),
            skills: std::collections::HashSet::new(),
            agents: std::collections::HashSet::new(),
            hooks: std::collections::HashSet::new(),
            mcp_servers: std::collections::HashSet::new(),
        },
    );
    assert!(kept.artifacts.is_empty());
}

#[test]
fn the_assembly_context_passes_through_untouched() {
    let mut input = candidate();
    input.marketplace_id = Some(systemprompt::identifiers::MarketplaceId::new("demo"));
    input.diagnostics.push("assembly warning".to_owned());
    let owners = input.artifact_owners.clone();

    let kept = retained(
        input,
        &EntryKeepSets {
            plugins: keep(&["demo-admin"], |s| PluginId::try_new(s).expect("plugin id")),
            skills: std::collections::HashSet::new(),
            agents: std::collections::HashSet::new(),
            hooks: std::collections::HashSet::new(),
            mcp_servers: std::collections::HashSet::new(),
        },
    );
    assert_eq!(kept.artifact_owners, owners);
    assert_eq!(
        kept.marketplace_id.map(|id| id.to_string()),
        Some("demo".to_owned())
    );
    assert_eq!(kept.diagnostics, vec!["assembly warning".to_owned()]);
}

#[test]
fn keeping_everything_is_the_identity() {
    let kept = retained(
        candidate(),
        &EntryKeepSets {
            plugins: keep(&["demo-admin", "demo-commons"], |s| PluginId::try_new(s).expect("plugin id")),
            skills: keep(&["skill-a", "skill-b"], |s| SkillId::try_new(s).expect("skill id")),
            agents: keep(&["agent-a"], |s| AgentId::new(s)),
            hooks: keep(&["hook-a"], |s| HookId::new(s)),
            mcp_servers: keep(&["files", "systemprompt"], |s| McpServerId::new(s)),
        },
    );
    assert_eq!(kept.plugins.len(), 2);
    assert_eq!(kept.skills.len(), 2);
    assert_eq!(kept.agents.len(), 1);
    assert_eq!(kept.hooks.len(), 1);
    assert_eq!(kept.managed_mcp_servers.len(), 2);
    assert_eq!(kept.artifacts.len(), 2);
}
