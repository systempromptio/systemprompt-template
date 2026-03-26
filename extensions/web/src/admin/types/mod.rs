pub mod access_control;
mod dashboard;
mod jobs;
mod marketplace;
mod marketplace_upload;
pub mod marketplaces;
mod plugins;
mod plugins_config;
mod plugins_requests;
mod user_context;
pub mod user_entities;
mod users;
mod webhook;

pub use dashboard::{
    AchievementInfo, ActivityStats, DashboardData, DashboardQuery, DepartmentActivity,
    DepartmentQuery, DepartmentScore, EventRow, EventTypeBreakdown, EventsQuery, EventsResponse,
    GovernanceDecisionRow, GovernanceEvent, HourlyActivity, LeaderboardEntry, McpAccessSummary,
    ModelUsage, PaginationQuery,
    ProjectActivity, SkillCount, TimeSeriesBucket, ToolSuccessRate, TopUser, UnlockedAchievement,
    UserGamificationProfile,
};
pub use jobs::JobSummary;
pub use marketplace::{
    MarketplacePlugin, MarketplaceQuery, PluginRating, PluginRatingAggregate, PluginUsageAggregate,
    PluginUser, SubmitRatingRequest, UpdateVisibilityRequest, VisibilityRule, VisibilityRuleInput,
};
pub use marketplace_upload::{
    AllVersionsSummaryRow, MarketplaceChangelogEntry, MarketplaceRestoreResponse,
    MarketplaceUploadResponse, MarketplaceVersion, MarketplaceVersionSummary, NewChangelogEntry,
    ParsedSkill, SyncDiff,
};
pub use plugins_config::{
    AgentDetail, AgentInfo, HookCatalogEntry, HookDetail, HookOverview, McpServerDetail,
    PlatformPluginConfig, PlatformPluginConfigFile, PluginDetail, PluginOnboardingConfig,
    PluginOnboardingDataSource, PluginOnboardingQuestion, PluginOverview, SkillInfo,
};
pub use plugins_requests::{
    CreateAgentRequest, CreateHookRequest, CreateMcpRequest, CreatePluginRequest, EnvVarEntry,
    ImportPluginRequest, UpdateAgentRequest, UpdateHookRequest, UpdateMcpRawYamlRequest,
    UpdateMcpRequest, UpdatePluginEnvRequest, UpdatePluginRequest, UpdatePluginSkillsRequest,
    UpdateSkillFileRequest, UserQuery,
};
pub use user_context::UserContext;

#[derive(Debug, Clone)]
pub struct MarketplaceContext {
    pub user_id: String,
    pub site_url: String,
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
}
pub use user_entities::{
    CreateUserMcpServerRequest, CreateUserPluginRequest, ForkAgentRequest, ForkHookRequest,
    ForkMcpServerRequest, ForkPluginRequest, ForkSkillRequest, UpdateUserMcpServerRequest,
    UpdateUserPluginRequest, UserMcpServer, UserPlugin, UserPluginWithAssociations,
};
pub use users::{
    AgentSkill, CreateSkillRequest, CreateUserAgentRequest, CreateUserHookRequest,
    CreateUserRequest, DepartmentStats, SkillSecret, UpdateSkillRequest, UpdateUserAgentRequest,
    UpdateUserHookRequest, UpdateUserRequest, UpdateUserSkillRequest, UpsertSkillSecretRequest,
    UsageEvent, UserAgent, UserDetail, UserHook, UserSkill, UserSummary, UsersQuery,
};
pub use webhook::{
    GovernQuery, HookEventPayload, StatusLinePayload, StatusLineQuery, TrackQuery,
    TranscriptPayload, TranscriptQuery,
};
