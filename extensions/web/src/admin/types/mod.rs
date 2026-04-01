pub mod access_control;
pub mod control_center;
pub mod conversation_analytics;
mod dashboard;
mod dashboard_enterprise;
mod dashboard_traffic;
pub mod hooks;
pub mod hooks_export;
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
pub mod webhook;

pub use dashboard::{
    AchievementInfo, ActivityStats, ContentPerformanceRow, DashboardData, DashboardQuery,
    DepartmentActivity, DepartmentQuery, DepartmentScore, EventBreakdown, EventRow,
    EventTypeBreakdown, EventsQuery, EventsResponse, GovernanceDecisionRow, GovernanceEvent, HourlyActivity,
    LeaderboardEntry, McpAccessEvent, McpAccessSummary, ModelUsage, PaginationQuery,
    ProjectActivity, TokenUsageRow,
    RealtimePulse, RecentMcpError, SkillCount, TimeSeriesBucket, ToolSuccessRate,
    TopPageDailyBucket, TopUser, TrafficCountryBucket, TrafficData, TrafficDevice, TrafficGeo,
    TrafficKpis, TrafficReadingPattern, TrafficSource, TrafficTimeBucket, TrafficTopPage,
    UnlockedAchievement, UserGamificationProfile,
};
pub use hooks::{
    CreateUserHookRequest, HookEventTypeStat, HookSummaryStats, HookTimeSeriesBucket, HooksQuery,
    UpdateUserHookRequest, UserHook,
};
pub use hooks_export::{HookEventType, HookHandler, HooksFile, HttpHook, MatcherGroup};
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
    AgentDetail, AgentInfo, AgentSkillInfo, HookCatalogEntry, HookDetail, HookOverview,
    McpServerDetail, PlatformPluginConfig, PlatformPluginConfigFile, PluginDetail,
    PluginOnboardingConfig, PluginOnboardingDataSource, PluginOnboardingQuestion, PluginOverview,
    RequiredSecret, SkillInfo,
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
    pub tier_name: String,
    pub is_premium: bool,
    pub rank_level: i32,
    pub rank_name: String,
    pub rank_tier: String,
    pub total_xp: i64,
    pub xp_progress_pct: f64,
    pub has_completed_onboarding: bool,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub next_rank_name: String,
    pub xp_to_next_rank: i64,
    pub plugin_token: String,
}
pub use conversation_analytics::{
    EntityEffectiveness, EntityUsageSummary, RateSessionRequest, RateSkillRequest,
    SessionEntityLink, SessionRating, SkillEffectiveness, SkillRating,
};
pub use user_entities::{
    CreateUserMcpServerRequest, CreateUserPluginRequest, ForkAgentRequest, ForkHookRequest,
    ForkMcpServerRequest, ForkPluginRequest, ForkSkillRequest, UpdateUserMcpServerRequest,
    UpdateUserPluginRequest, UserMcpServer, UserPlugin, UserPluginWithAssociations,
};
pub use users::{
    AgentSkill, ContentBytes, CookieSession, CreateSkillRequest, CreateUserAgentRequest,
    CreateUserRequest, DepartmentStats, DetectedEntity, EventTypeCount, JwtIdentity, SkillSecret,
    ToolUsageCount, UpdateSkillRequest, UpdateUserAgentRequest, UpdateUserRequest,
    UpdateUserSkillRequest, UpsertSkillSecretRequest, UsageEvent, UserAgent, UserBasicInfo,
    UserDetail, UserPluginCounts, UserSession, UserSkill, UserSummary, UserTier, UsersQuery,
};
pub use webhook::{
    GovernQuery, HookEventPayload, StatusLinePayload, StatusLineQuery, TrackQuery,
    TranscriptPayload, TranscriptQuery,
};
