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

use systemprompt::marketplace::MarketplaceCandidate;
use systemprompt::models::bridge::ids::{LibraryArtifactId, PluginId};
use systemprompt::models::bridge::manifest::{
    AgentEntry, ArtifactEntry, HookEntry, ManagedMcpServer, PluginEntry, SkillEntry,
};
use systemprompt_security::authz::{EntityKind, EntityRef};
use systemprompt_web_admin::authz::department::{department_dimension, department_rule_type};
use systemprompt_web_admin::marketplace_filter::keepsets::{
    CandidateEntityIds, KeepSet, KeepSets, apply_keep_sets, entity_ref_for,
};

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
    let kinds = [
        EntityKind::Plugin,
        EntityKind::Skill,
        EntityKind::Agent,
        EntityKind::McpServer,
        EntityKind::Marketplace,
        EntityKind::GatewayRoute,
        EntityKind::Hook,
        EntityKind::SlackWorkspace,
        EntityKind::SlackChannel,
        EntityKind::TeamsTenant,
        EntityKind::TeamsConversation,
    ];
    let mut seen = Vec::new();
    for kind in kinds {
        let entity = entity_ref_for(kind, "id-1");
        let rendered = format!("{entity:?}");
        assert!(
            rendered.contains("id-1"),
            "{kind:?} dropped the id: {rendered}"
        );
        assert!(
            !seen.contains(&std::mem::discriminant(&entity)),
            "{kind:?} reuses another kind's EntityRef variant"
        );
        seen.push(std::mem::discriminant(&entity));
    }
    assert!(matches!(
        entity_ref_for(EntityKind::Plugin, "p"),
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
    MarketplaceCandidate::new(
        vec![plugin_entry("demo-admin"), plugin_entry("demo-commons")],
        vec![skill_entry("skill-a"), skill_entry("skill-b")],
        vec![agent_entry("agent-a")],
        vec![hook_entry("hook-a")],
        vec![mcp_entry("files"), mcp_entry("systemprompt")],
        vec![artifact_entry("art-admin"), artifact_entry("art-common")],
    )
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

fn keep(ids: &[&str]) -> KeepSet {
    ids.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn candidate_ids_are_read_off_every_entry_list() {
    let ids = CandidateEntityIds::from_candidate(&candidate());
    assert_eq!(ids.plugins, ["demo-admin", "demo-commons"]);
    assert_eq!(ids.skills, ["skill-a", "skill-b"]);
    assert_eq!(ids.agents, ["agent-a"]);
    assert_eq!(ids.hooks, ["hook-a"]);
    // Why: an MCP server is keyed by its name, not an id field.
    assert_eq!(ids.mcp, ["files", "systemprompt"]);
}

#[test]
fn keep_sets_shrink_every_list_to_what_survived() {
    let kept = apply_keep_sets(
        candidate(),
        &KeepSets {
            plugins: keep(&["demo-commons"]),
            skills: keep(&["skill-b"]),
            agents: KeepSet::new(),
            hooks: keep(&["hook-a"]),
            mcp: keep(&["files"]),
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
    let kept = apply_keep_sets(
        candidate(),
        &KeepSets {
            plugins: keep(&["demo-commons"]),
            skills: KeepSet::new(),
            agents: KeepSet::new(),
            hooks: KeepSet::new(),
            mcp: KeepSet::new(),
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
    let kept = apply_keep_sets(
        candidate(),
        &KeepSets {
            plugins: KeepSet::new(),
            skills: keep(&["skill-a"]),
            agents: KeepSet::new(),
            hooks: KeepSet::new(),
            mcp: KeepSet::new(),
        },
    );
    assert!(kept.artifacts.is_empty());
    assert_eq!(kept.skills.len(), 1, "other lists are unaffected");
}

#[test]
fn an_unowned_artifact_is_dropped_rather_than_defaulting_to_visible() {
    let mut input = candidate();
    input.artifact_owners.clear();
    let kept = apply_keep_sets(
        input,
        &KeepSets {
            plugins: keep(&["demo-admin", "demo-commons"]),
            skills: KeepSet::new(),
            agents: KeepSet::new(),
            hooks: KeepSet::new(),
            mcp: KeepSet::new(),
        },
    );
    assert!(kept.artifacts.is_empty());
}

#[test]
fn the_owner_map_and_marketplace_scope_pass_through_untouched() {
    let mut input = candidate();
    input.marketplace_id = Some(systemprompt::identifiers::MarketplaceId::new("demo"));
    let owners = input.artifact_owners.clone();

    let kept = apply_keep_sets(
        input,
        &KeepSets {
            plugins: keep(&["demo-admin"]),
            skills: KeepSet::new(),
            agents: KeepSet::new(),
            hooks: KeepSet::new(),
            mcp: KeepSet::new(),
        },
    );
    assert_eq!(kept.artifact_owners, owners);
    assert_eq!(
        kept.marketplace_id.map(|id| id.to_string()),
        Some("demo".to_owned())
    );
}

#[test]
fn keeping_everything_is_the_identity() {
    let kept = apply_keep_sets(
        candidate(),
        &KeepSets {
            plugins: keep(&["demo-admin", "demo-commons"]),
            skills: keep(&["skill-a", "skill-b"]),
            agents: keep(&["agent-a"]),
            hooks: keep(&["hook-a"]),
            mcp: keep(&["files", "systemprompt"]),
        },
    );
    assert_eq!(kept.plugins.len(), 2);
    assert_eq!(kept.skills.len(), 2);
    assert_eq!(kept.agents.len(), 1);
    assert_eq!(kept.hooks.len(), 1);
    assert_eq!(kept.managed_mcp_servers.len(), 2);
    assert_eq!(kept.artifacts.len(), 2);
}
