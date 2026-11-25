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
    &datasets::people::Groups,
    &datasets::projects::Projects,
    &datasets::history::History,
    &datasets::history::OrgHistory,
    &datasets::dashboard::ModelStats,
    &datasets::dashboard::SkillStats,
    &datasets::dashboard::ToolStats,
    &datasets::dashboard::ToolServers,
    &datasets::dashboard::SessionCosts,
    &datasets::dashboard_cost::ProviderCosts,
    &datasets::dashboard_cost::ProviderCostByDay,
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
    #[cfg(feature = "mcp")]
    &datasets::versions::Marketplaces,
    #[cfg(feature = "mcp")]
    &datasets::versions::Versions,
    #[cfg(feature = "mcp")]
    &datasets::plugin_eval::PluginEval,
    #[cfg(feature = "mcp")]
    &datasets::plugin_eval::PluginEvalTools,
    &datasets::governance::Decisions,
    &datasets::governance::Findings,
    &datasets::governance::Secrets,
    &datasets::reports::CustomerUsers,
    &datasets::reports::CustomerProjects,
    &datasets::reports::CustomerModels,
    &datasets::reports::InternalProviders,
    &datasets::reports::InternalModels,
];

pub(crate) fn find(id: &str) -> Option<&'static dyn DataSet> {
    ALL.iter().copied().find(|d| d.id() == id)
}
