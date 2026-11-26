//! Every exportable table, by id.

use super::datasets;
use super::model::DataSet;

// Why: a static slice rather than `inventory` — the set is closed, small, and
// a reader should be able to see every export the console offers in one
// place. Order is the order the dialog lists them in when a page offers more
// than one.
pub(crate) const ALL: &[&dyn DataSet] = &[
    #[cfg(feature = "gateway")]
    &datasets::requests::Requests,
    #[cfg(feature = "agents")]
    &datasets::sessions::Sessions,
    #[cfg(feature = "agents")]
    &datasets::traces::Traces,
    #[cfg(feature = "agents")]
    &datasets::tools::Tools,
    #[cfg(feature = "agents")]
    &datasets::tools::ToolsBreakdown,
    #[cfg(feature = "agents")]
    &datasets::tools::Artifacts,
    #[cfg(feature = "agents")]
    &datasets::tools::ArtifactsBreakdown,
    #[cfg(feature = "agents")]
    &datasets::contexts::Contexts,
    #[cfg(feature = "agents")]
    &datasets::contexts::People,
    &datasets::people::Users,
    #[cfg(feature = "gateway")]
    &datasets::people::Groups,
    #[cfg(feature = "mcp")]
    &datasets::projects::Projects,
    #[cfg(feature = "agents")]
    &datasets::history::History,
    #[cfg(feature = "agents")]
    &datasets::history::OrgHistory,
    #[cfg(feature = "gateway")]
    &datasets::dashboard::ModelStats,
    #[cfg(feature = "mcp")]
    &datasets::dashboard_tools::SkillStats,
    #[cfg(feature = "mcp")]
    &datasets::dashboard_tools::ToolStats,
    #[cfg(feature = "mcp")]
    &datasets::dashboard_tools::ToolServers,
    #[cfg(feature = "gateway")]
    &datasets::dashboard::SessionCosts,
    #[cfg(feature = "gateway")]
    &datasets::dashboard_cost::ProviderCosts,
    #[cfg(feature = "gateway")]
    &datasets::dashboard_cost::ProviderCostByDay,
    #[cfg(feature = "gateway")]
    &datasets::dashboard_cost::ContainerUsage,
    #[cfg(feature = "agents")]
    &datasets::analysis_skills::Skills,
    #[cfg(feature = "agents")]
    &datasets::analysis_skill_conversations::SkillConversations,
    #[cfg(feature = "agents")]
    &datasets::analysis_conversations::ClassifiedConversations,
    #[cfg(feature = "agents")]
    &datasets::analysis_conversations::Breakdown,
    #[cfg(feature = "agents")]
    &datasets::analysis_conversation::ConversationTurns,
    #[cfg(feature = "agents")]
    &datasets::analysis_skill_runs::SkillRuns,
    #[cfg(feature = "agents")]
    &datasets::analysis_skill_runs::KitReleaseImpact,
    #[cfg(feature = "agents")]
    &datasets::versions::Marketplaces,
    #[cfg(feature = "agents")]
    &datasets::versions::Versions,
    #[cfg(feature = "agents")]
    &datasets::plugin_eval::PluginEval,
    #[cfg(feature = "agents")]
    &datasets::plugin_eval::PluginEvalTools,
    &datasets::governance::Decisions,
    #[cfg(feature = "gateway")]
    &datasets::governance::Findings,
    &datasets::governance::Secrets,
    #[cfg(feature = "gateway")]
    &datasets::reports::CustomerUsers,
    #[cfg(feature = "gateway")]
    &datasets::reports::CustomerProjects,
    #[cfg(feature = "gateway")]
    &datasets::reports::CustomerModels,
    #[cfg(feature = "gateway")]
    &datasets::reports::InternalProviders,
    #[cfg(feature = "gateway")]
    &datasets::reports::InternalModels,
];

pub(crate) fn find(id: &str) -> Option<&'static dyn DataSet> {
    ALL.iter().copied().find(|d| d.id() == id)
}
