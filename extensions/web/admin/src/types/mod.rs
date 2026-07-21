//! Value types for the admin plane, grouped by the surface that owns them.

pub mod access_control;
pub mod constants;
pub mod control_center;
pub mod conversation_analytics;
mod dashboard;
mod dashboard_enterprise;
pub mod departments;
pub mod gateway;
pub mod hooks_export;
mod jobs;
mod plugins;
mod plugins_config;
mod plugins_requests;
mod traffic;
mod user_context;
pub use departments::{Department, DepartmentInput, DepartmentMember, DepartmentSummary};
mod users;
pub mod webhook;

pub use dashboard::{
    AchievementInfo, ActivityStats, ContentPerformanceRow, DashboardData, DashboardQuery,
    DepartmentActivity, DepartmentQuery, DepartmentScore, EventBreakdown, EventFeedRow,
    EventTypeBreakdown, EventsQuery, EventsResponse, GovernanceDecisionRow, GovernanceEvent,
    HourlyActivity, IncidentGroup, LeaderboardEntry, McpAccessEvent, McpAccessSummary, ModelUsage,
    PaginationQuery, ProjectActivity, RealtimePulse, RecentMcpError, SkillCount, TimeSeriesBucket,
    TokenUsageRow, ToolSuccessRate, TopActor, TopPageDailyBucket, TopPolicy, TopUser,
    TrafficCountryBucket, TrafficData, TrafficDevice, TrafficGeo, TrafficKpis,
    TrafficReadingPattern, TrafficSource, TrafficTimeBucket, TrafficTopPage, UnlockedAchievement,
    UserGamificationProfile, WindowedCounts,
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
pub use constants::{
    ACTION_GRANTED, CATEGORY_AI_SESSIONS, CATEGORY_EDITS, DECISION_DENY, DIR_PYCACHE, ENTITY_AGENT,
    ENTITY_MARKETPLACE, ENTITY_MCP_SERVER, ENTITY_MCP_TOOL, ENTITY_PLUGIN, ENTITY_SKILL,
    EVENT_POST_TOOL_USE, EVENT_POST_TOOL_USE_FAILURE, EVENT_SESSION_END, EVENT_SESSION_START,
    EVENT_STOP, GIT_HEAD, GIT_INFO_REFS, GIT_UPLOAD_PACK, HOOK_TYPE_HTTP, IMPORT_TARGET_USER,
    LOG_CONTEXT_GITHUB, MCP_CONFIG_PATH, PERMISSION_MODE_PLAN, PLUGIN_ID_SYSTEMPROMPT,
    PLUGIN_MANIFEST_PATH, RANGE_7D, RANGE_14D, RANGE_24H, ROLE_ADMIN, SCRIPT_SOURCE_TRACKING,
    SERVER_TYPE_EXTERNAL, SERVER_TYPE_INTERNAL, SKILL_FILENAME, SOURCE_CUSTOM, SOURCE_USER,
    STATUS_ACTIVE, STATUS_DELETED, TAB_GOVERNANCE, TAB_MCP, TAB_REPORT, TRAFFIC_RANGE_30D,
    TRAFFIC_RANGE_TODAY, TRAFFIC_RANGE_YESTERDAY,
};
pub use conversation_analytics::{
    EntityEffectiveness, EntityUsageSummary, RateSessionRequest, RateSkillRequest,
    SessionEntityLink, SessionRating, SkillEffectiveness, SkillRating,
};
pub use users::{
    ContentBytes, CookieSession, CreateUserRequest, DepartmentStats, DetectedEntity,
    EventTypeCount, JwtIdentity, SkillSecret, ToolUsageCount, UpdateUserRequest,
    UpsertSkillSecretRequest, UserBasicInfo, UserDetail, UserIdentityRow, UserSession, UserSummary,
    UserTier, UserUsageEvent, UsersQuery,
};
pub use webhook::{
    GovernQuery, HookEventPayload, StatusLinePayload, StatusLineQuery, TrackQuery,
    TranscriptPayload, TranscriptQuery,
};
