pub mod access_control;
pub mod constants;
pub mod control_center;
pub mod conversation_analytics;
mod dashboard;
mod dashboard_enterprise;
mod dashboard_traffic;
pub mod gateway;
pub mod hooks_export;
mod jobs;
mod plugins;
mod plugins_config;
mod plugins_requests;
mod user_context;
pub mod departments;
pub use departments::{Department, DepartmentInput, DepartmentMember, DepartmentSummary};
mod users;
pub mod webhook;

pub use dashboard::{
    AchievementInfo, ActivityStats, ContentPerformanceRow, DashboardData, DashboardQuery,
    DepartmentActivity, DepartmentQuery, DepartmentScore, EventBreakdown, EventRow,
    EventTypeBreakdown, EventsQuery, EventsResponse, GovernanceDecisionRow, GovernanceEvent,
    HourlyActivity, IncidentGroup, LeaderboardEntry, McpAccessEvent, McpAccessSummary, ModelUsage,
    PaginationQuery, ProjectActivity, RealtimePulse, RecentMcpError, SkillCount, TimeSeriesBucket,
    TokenUsageRow, ToolSuccessRate, TopActor, TopPageDailyBucket, TopPolicy, TopUser,
    TrafficCountryBucket, TrafficData,
    TrafficDevice, TrafficGeo, TrafficKpis, TrafficReadingPattern, TrafficSource,
    TrafficTimeBucket, TrafficTopPage, UnlockedAchievement, UserGamificationProfile,
    WindowedCounts,
};
pub use gateway::{
    GatewayConfigView, GatewayRouteView, ReorderRoutesRequest, UpdateGatewaySettingsRequest,
};
pub use hooks_export::{HookEventType, HookHandler, HooksFile, HttpHook, MatcherGroup};
pub use jobs::JobSummary;
pub use plugins_config::{
    AgentCatalogEntry, AgentDetail, AgentInfo, AgentSkillInfo, ConfiguredHook, HookCatalogEntry,
    HookDetail, HookOverview, McpServerDetail, PlatformPluginConfig, PluginDetail,
    PluginOnboardingConfig, PluginOnboardingDataSource, PluginOnboardingQuestion, PluginOverview,
    RequiredSecret, SkillCatalogEntry, SkillInfo,
};
pub use plugins_requests::{
    CreateAgentRequest, CreateHookRequest, CreateMcpRequest, CreatePluginRequest, EnvVarEntry,
    ImportPluginRequest, UpdateAgentRequest, UpdateHookRequest, UpdateMcpRawYamlRequest,
    UpdateMcpRequest, UpdatePluginEnvRequest, UpdatePluginRequest, UpdatePluginSkillsRequest,
    UpdateSkillFileRequest, UserQuery,
};
pub use user_context::UserContext;

#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct IdQuery {
    #[serde(default)]
    pub id: Option<String>,
}

impl IdQuery {
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct MarketplaceContext {
    pub user_id: systemprompt_web_shared::UserId,
    pub site_url: String,
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
    pub rank_level: i32,
    pub rank_name: String,
    pub rank_tier: systemprompt_web_shared::RankTier,
    pub total_xp: i64,
    pub xp_progress_pct: f64,
    pub has_completed_onboarding: bool,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub next_rank_name: String,
    pub xp_to_next_rank: i64,
}
pub use constants::*;
pub use conversation_analytics::{
    EntityEffectiveness, EntityUsageSummary, RateSessionRequest, RateSkillRequest,
    SessionEntityLink, SessionRating, SkillEffectiveness, SkillRating,
};
pub use users::{
    ContentBytes, CookieSession, CreateUserRequest, DepartmentStats, DetectedEntity,
    EventTypeCount, JwtIdentity, SkillSecret, ToolUsageCount, UpdateUserRequest,
    UpsertSkillSecretRequest, UsageEvent, UserBasicInfo, UserDetail, UserIdentityRow,
    UserSession, UserSummary, UserTier, UsersQuery,
};
pub use webhook::{
    GovernQuery, HookEventPayload, StatusLinePayload, StatusLineQuery, TrackQuery,
    TranscriptPayload, TranscriptQuery,
};
