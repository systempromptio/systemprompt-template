//! `TemplateMarketplaceFilter` against a live Postgres.
//!
//! The pure keep-set plumbing is exercised in the admin crate's own
//! `authz_marketplace_pure` test; what only a database can show is the query
//! flow around it — the principal lookup, the per-kind rule fetch, the
//! marketplace parent, and the default-included fallbacks.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use systemprompt::database::Database;
use systemprompt::identifiers::{MarketplaceId, UserId};
use systemprompt::marketplace::{MarketplaceCandidate, MarketplaceFilter, MarketplaceFilterError};
use systemprompt::models::bridge::ids::{LibraryArtifactId, PluginId};
use systemprompt::models::bridge::manifest::{
    AgentEntry, ArtifactEntry, HookEntry, ManagedMcpServer, PluginEntry, SkillEntry,
};
use systemprompt::models::services::MarketplaceAccess;
use systemprompt_security::authz::{
    Access, AccessControlRepository, EntityKind, RuleType, UpsertRuleParams,
};
use systemprompt_web_admin::marketplace_filter::TemplateMarketplaceFilter;

use crate::fixtures::{insert_user_with_roles, unique};
use crate::tempdb::TempDb;

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

struct Harness {
    db: TempDb,
    filter: Arc<dyn MarketplaceFilter>,
    repo: AccessControlRepository,
}

impl Harness {
    async fn create() -> Option<Self> {
        let db = TempDb::create().await?;
        let database: systemprompt::database::DbPool = Arc::new(Database::from_pools(
            Arc::clone(&db.pool),
            Some(Arc::clone(&db.pool)),
        ));
        let filter = TemplateMarketplaceFilter::from_db(&database).expect("build filter");
        let repo = AccessControlRepository::from_pool(Arc::clone(&db.pool));
        Some(Self { db, filter, repo })
    }

    async fn entity(&self, kind: EntityKind, id: &str, default_included: bool) {
        self.repo
            .upsert_entity(kind, id, default_included, "test")
            .await
            .expect("upsert entity");
    }

    async fn rule(&self, kind: EntityKind, id: &str, role: &str, access: Access) {
        self.repo
            .upsert_rule(UpsertRuleParams {
                entity_type: kind,
                entity_id: id,
                rule_type: RuleType::ROLE,
                rule_value: role,
                access,
                justification: None,
            })
            .await
            .expect("upsert rule");
    }

    async fn user(&self, roles: &[&str]) -> UserId {
        let id = unique("mpf-user");
        let roles: Vec<String> = roles.iter().map(|r| (*r).to_owned()).collect();
        insert_user_with_roles(&self.db.pool, &id, &roles).await;
        UserId::new(id)
    }
}

#[tokio::test]
async fn a_user_with_no_row_is_rejected_rather_than_filtered_to_nothing() {
    let Some(h) = Harness::create().await else {
        return;
    };

    let err = h
        .filter
        .filter(
            &UserId::new(unique("ghost")),
            MarketplaceCandidate::default(),
        )
        .await
        .expect_err("unknown user must not resolve");

    assert!(
        matches!(err, MarketplaceFilterError::UnknownUser(_)),
        "expected UnknownUser, got {err:?}"
    );

    h.db.cleanup().await;
}

#[tokio::test]
async fn an_empty_candidate_round_trips_without_touching_the_rule_tables() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["user"]).await;

    let kept = h
        .filter
        .filter(&user, MarketplaceCandidate::default())
        .await
        .expect("filter empty candidate");

    assert!(kept.plugins.is_empty());
    assert!(kept.skills.is_empty());
    assert!(kept.agents.is_empty());
    assert!(kept.hooks.is_empty());
    assert!(kept.managed_mcp_servers.is_empty());

    h.db.cleanup().await;
}

#[tokio::test]
async fn an_entity_with_no_rule_and_no_catalog_row_is_dropped() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["user"]).await;
    let plugin = unique("plugin");

    let kept = h
        .filter
        .filter(
            &user,
            MarketplaceCandidate::new(
                vec![plugin_entry(&plugin)],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        )
        .await
        .expect("filter");

    assert!(
        kept.plugins.is_empty(),
        "default policy is explicit allow: an unregistered plugin is not visible"
    );

    h.db.cleanup().await;
}

#[tokio::test]
async fn a_default_included_catalog_row_makes_an_entity_visible_to_everyone() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["user"]).await;
    let plugin = unique("plugin");
    h.entity(EntityKind::Plugin, &plugin, true).await;

    let kept = h
        .filter
        .filter(
            &user,
            MarketplaceCandidate::new(
                vec![plugin_entry(&plugin)],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        )
        .await
        .expect("filter");

    assert_eq!(kept.plugins.len(), 1);
    assert_eq!(kept.plugins[0].id.as_str(), plugin);

    h.db.cleanup().await;
}

#[tokio::test]
async fn a_role_rule_admits_only_the_role_it_names() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let plugin = unique("plugin");
    h.entity(EntityKind::Plugin, &plugin, false).await;
    h.rule(EntityKind::Plugin, &plugin, "admin", Access::Allow)
        .await;

    let admin = h.user(&["admin"]).await;
    let plain = h.user(&["user"]).await;
    let candidate = || {
        MarketplaceCandidate::new(
            vec![plugin_entry(&plugin)],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        )
    };

    let for_admin = h.filter.filter(&admin, candidate()).await.expect("admin");
    let for_plain = h.filter.filter(&plain, candidate()).await.expect("plain");

    assert_eq!(for_admin.plugins.len(), 1);
    assert!(for_plain.plugins.is_empty());

    h.db.cleanup().await;
}

#[tokio::test]
async fn every_entity_kind_is_resolved_against_its_own_rules() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["staff"]).await;

    let plugin = unique("plugin");
    let skill_yes = unique("skill-yes");
    let skill_no = unique("skill-no");
    let agent = unique("agent");
    let hook = unique("hook");
    let mcp = unique("mcp");
    for (kind, id, allow) in [
        (EntityKind::Plugin, &plugin, true),
        (EntityKind::Skill, &skill_yes, true),
        (EntityKind::Skill, &skill_no, false),
        (EntityKind::Agent, &agent, true),
        (EntityKind::Hook, &hook, true),
        (EntityKind::McpServer, &mcp, true),
    ] {
        h.entity(kind, id, false).await;
        h.rule(
            kind,
            id,
            "staff",
            if allow { Access::Allow } else { Access::Deny },
        )
        .await;
    }

    let kept = h
        .filter
        .filter(
            &user,
            MarketplaceCandidate::new(
                vec![plugin_entry(&plugin)],
                vec![skill_entry(&skill_yes), skill_entry(&skill_no)],
                vec![agent_entry(&agent)],
                vec![hook_entry(&hook)],
                vec![mcp_entry(&mcp)],
                vec![],
            ),
        )
        .await
        .expect("filter");

    assert_eq!(kept.plugins.len(), 1);
    assert_eq!(kept.skills.len(), 1, "the denied skill is dropped");
    assert_eq!(kept.skills[0].id.as_str(), skill_yes);
    assert_eq!(kept.agents.len(), 1);
    assert_eq!(kept.hooks.len(), 1);
    assert_eq!(kept.managed_mcp_servers.len(), 1);

    h.db.cleanup().await;
}

#[tokio::test]
async fn one_marketplace_rule_covers_members_that_declare_none_of_their_own() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["staff"]).await;
    let marketplace = unique("mp");
    let skill = unique("skill");
    h.entity(EntityKind::Marketplace, &marketplace, false).await;
    h.rule(
        EntityKind::Marketplace,
        &marketplace,
        "staff",
        Access::Allow,
    )
    .await;

    let kept = h
        .filter
        .filter(
            &user,
            MarketplaceCandidate::new(
                vec![],
                vec![skill_entry(&skill)],
                vec![],
                vec![],
                vec![],
                vec![],
            )
            .with_marketplace(MarketplaceId::new(marketplace.clone()), None),
        )
        .await
        .expect("filter");

    assert_eq!(
        kept.skills.len(),
        1,
        "the marketplace parent grants a member with no rules of its own"
    );
    assert_eq!(
        kept.marketplace_id.map(|id| id.to_string()),
        Some(marketplace)
    );

    h.db.cleanup().await;
}

#[tokio::test]
async fn a_member_that_declares_a_rule_owns_its_decision_over_the_marketplace() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["staff"]).await;
    let marketplace = unique("mp");
    let skill = unique("skill");
    h.entity(EntityKind::Marketplace, &marketplace, false).await;
    h.rule(
        EntityKind::Marketplace,
        &marketplace,
        "staff",
        Access::Allow,
    )
    .await;
    h.entity(EntityKind::Skill, &skill, false).await;
    h.rule(EntityKind::Skill, &skill, "staff", Access::Deny)
        .await;

    let kept = h
        .filter
        .filter(
            &user,
            MarketplaceCandidate::new(
                vec![],
                vec![skill_entry(&skill)],
                vec![],
                vec![],
                vec![],
                vec![],
            )
            .with_marketplace(MarketplaceId::new(marketplace), None),
        )
        .await
        .expect("filter");

    assert!(
        kept.skills.is_empty(),
        "a member deny must win over the marketplace allow"
    );

    h.db.cleanup().await;
}

#[tokio::test]
async fn the_candidate_access_supplies_the_marketplace_default_when_no_catalog_row_exists() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["user"]).await;
    let marketplace = unique("mp");
    let skill = unique("skill");
    let access = MarketplaceAccess {
        default_included: true,
        roles: vec![],
        attributes: BTreeMap::new(),
        justification: None,
    };

    let kept = h
        .filter
        .filter(
            &user,
            MarketplaceCandidate::new(
                vec![],
                vec![skill_entry(&skill)],
                vec![],
                vec![],
                vec![],
                vec![],
            )
            .with_marketplace(MarketplaceId::new(marketplace), Some(access)),
        )
        .await
        .expect("filter");

    assert_eq!(
        kept.skills.len(),
        1,
        "an unregistered marketplace falls back to the candidate's declared access"
    );
    assert!(kept.access.is_some(), "the access block is carried through");

    h.db.cleanup().await;
}

#[tokio::test]
async fn a_salesforce_rule_admits_only_users_with_a_linked_identity() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let mcp = unique("sf-mcp");
    h.entity(EntityKind::McpServer, &mcp, false).await;
    h.repo
        .upsert_rule(UpsertRuleParams {
            entity_type: EntityKind::McpServer,
            entity_id: &mcp,
            rule_type: systemprompt_web_admin::authz::salesforce::salesforce_rule_type(),
            rule_value: systemprompt_web_admin::authz::salesforce::SALESFORCE_LINKED_VALUE,
            access: Access::Allow,
            justification: None,
        })
        .await
        .expect("upsert salesforce rule");

    let linked = h.user(&["user"]).await;
    let unlinked = h.user(&["user"]).await;
    systemprompt_web_admin::repositories::users::salesforce_identity::upsert(
        &h.db.pool,
        &linked,
        "linked.user@example.test",
    )
    .await
    .expect("link salesforce identity");

    let candidate = || {
        MarketplaceCandidate::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![mcp_entry(&mcp)],
            vec![],
        )
    };

    let for_linked = h.filter.filter(&linked, candidate()).await.expect("linked");
    let for_unlinked = h
        .filter
        .filter(&unlinked, candidate())
        .await
        .expect("unlinked");

    assert_eq!(
        for_linked.managed_mcp_servers.len(),
        1,
        "a linked identity row grants the salesforce-gated server"
    );
    assert!(
        for_unlinked.managed_mcp_servers.is_empty(),
        "a passkey-only user matches no salesforce rule and the default is closed"
    );

    h.db.cleanup().await;
}

#[tokio::test]
async fn artifacts_follow_the_plugin_decision_the_database_produced() {
    let Some(h) = Harness::create().await else {
        return;
    };
    let user = h.user(&["user"]).await;
    let kept_plugin = unique("plugin-keep");
    let dropped_plugin = unique("plugin-drop");
    h.entity(EntityKind::Plugin, &kept_plugin, true).await;
    h.entity(EntityKind::Plugin, &dropped_plugin, false).await;

    let owners = BTreeMap::from([
        (
            LibraryArtifactId::try_new("art-keep").expect("artifact id"),
            BTreeSet::from([PluginId::try_new(&kept_plugin).expect("plugin id")]),
        ),
        (
            LibraryArtifactId::try_new("art-drop").expect("artifact id"),
            BTreeSet::from([PluginId::try_new(&dropped_plugin).expect("plugin id")]),
        ),
    ]);

    let kept = h
        .filter
        .filter(
            &user,
            MarketplaceCandidate::new(
                vec![plugin_entry(&kept_plugin), plugin_entry(&dropped_plugin)],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![artifact_entry("art-keep"), artifact_entry("art-drop")],
            )
            .with_artifact_owners(owners),
        )
        .await
        .expect("filter");

    let artifacts: Vec<_> = kept.artifacts.iter().map(|a| a.id.to_string()).collect();
    assert_eq!(kept.plugins.len(), 1);
    assert_eq!(artifacts, ["art-keep"]);

    h.db.cleanup().await;
}
