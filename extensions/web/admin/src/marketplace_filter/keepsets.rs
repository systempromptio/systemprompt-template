//! Pure plumbing for [`super::TemplateMarketplaceFilter`]: entity-ref mapping,
//! candidate id extraction, and keep-set application. None of this touches the
//! database — it is the deterministic shape-shuffling around the access-control
//! resolver, split out to keep the filter module focused on the query flow.

use systemprompt::identifiers::{
    AgentId, HookId, MarketplaceId, McpServerId, PluginId, RouteId, SkillId, SlackChannelId,
    SlackWorkspaceId, TeamsConversationId, TeamsTenantId,
};
use systemprompt::marketplace::MarketplaceCandidate;
use systemprompt_security::authz::{EntityKind, EntityRef, ResolveParent};

pub(crate) fn entity_ref_for(kind: EntityKind, id: &str) -> EntityRef {
    match kind {
        EntityKind::Plugin => EntityRef::Plugin(PluginId::new(id)),
        EntityKind::Skill => EntityRef::Skill(SkillId::new(id)),
        EntityKind::Agent => EntityRef::Agent(AgentId::new(id)),
        EntityKind::McpServer => EntityRef::McpServer(McpServerId::new(id)),
        EntityKind::Marketplace => EntityRef::Marketplace(MarketplaceId::new(id)),
        EntityKind::GatewayRoute => EntityRef::GatewayRoute(RouteId::new(id)),
        EntityKind::Hook => EntityRef::Hook(HookId::new(id)),
        EntityKind::SlackWorkspace => EntityRef::SlackWorkspace(SlackWorkspaceId::new(id)),
        EntityKind::SlackChannel => EntityRef::SlackChannel(SlackChannelId::new(id)),
        EntityKind::TeamsTenant => EntityRef::TeamsTenant(TeamsTenantId::new(id)),
        EntityKind::TeamsConversation => EntityRef::TeamsConversation(TeamsConversationId::new(id)),
    }
}

pub(super) struct CandidateEntityIds {
    pub plugins: Vec<String>,
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    pub hooks: Vec<String>,
    pub mcp: Vec<String>,
}

impl CandidateEntityIds {
    pub(super) fn from_candidate(candidate: &MarketplaceCandidate) -> Self {
        Self {
            plugins: candidate.plugins.iter().map(|p| p.id.to_string()).collect(),
            skills: candidate.skills.iter().map(|s| s.id.to_string()).collect(),
            agents: candidate.agents.iter().map(|a| a.id.to_string()).collect(),
            hooks: candidate.hooks.iter().map(|h| h.id.to_string()).collect(),
            mcp: candidate
                .managed_mcp_servers
                .iter()
                .map(|m| m.name.to_string())
                .collect(),
        }
    }
}

pub(super) type KeepSet = std::collections::HashSet<String>;

pub(super) struct KeepIdsQuery<'a> {
    pub user_id: &'a str,
    pub roles: &'a [String],
    pub kind: EntityKind,
    pub ids: &'a [String],
    pub parents: &'a [ResolveParent<'a>],
}

pub(super) struct KeepSets {
    pub plugins: KeepSet,
    pub skills: KeepSet,
    pub agents: KeepSet,
    pub hooks: KeepSet,
    pub mcp: KeepSet,
}

pub(super) fn apply_keep_sets(
    candidate: MarketplaceCandidate,
    keep: &KeepSets,
) -> MarketplaceCandidate {
    MarketplaceCandidate {
        plugins: candidate
            .plugins
            .into_iter()
            .filter(|p| keep.plugins.contains(p.id.as_str()))
            .collect(),
        skills: candidate
            .skills
            .into_iter()
            .filter(|s| keep.skills.contains(s.id.as_str()))
            .collect(),
        agents: candidate
            .agents
            .into_iter()
            .filter(|a| keep.agents.contains(a.id.as_str()))
            .collect(),
        hooks: candidate
            .hooks
            .into_iter()
            .filter(|h| keep.hooks.contains(h.id.as_str()))
            .collect(),
        managed_mcp_servers: candidate
            .managed_mcp_servers
            .into_iter()
            .filter(|m| keep.mcp.contains(m.name.as_str()))
            .collect(),
        // Artifacts carry no access rule of their own, so they inherit their
        // owning plugins' decision: an artifact is staged only while at least
        // one plugin that ships it survived the plugin keep-set. Without this
        // an admin-only dashboard would be staged to every user's Artifacts
        // library even though its plugin was filtered out.
        artifacts: candidate
            .artifacts
            .into_iter()
            .filter(|a| {
                candidate
                    .artifact_owners
                    .get(&a.id)
                    .is_some_and(|owners| owners.iter().any(|p| keep.plugins.contains(p.as_str())))
            })
            .collect(),
        artifact_owners: candidate.artifact_owners,
        // Carry the owning marketplace context through unchanged; the
        // filter only shrinks entry lists, it must not drop the scope the
        // gateway attached.
        marketplace_id: candidate.marketplace_id,
        access: candidate.access,
    }
}
