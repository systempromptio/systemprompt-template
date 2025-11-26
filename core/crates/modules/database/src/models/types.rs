use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;

/// Type alias for database rows represented as JSON objects.
///
/// Each row is a `HashMap` where keys are column names and values are JSON values.
pub type JsonRow = HashMap<String, serde_json::Value>;

/// Parse datetime from `PostgreSQL` database value.
///
/// `PostgreSQL` stores datetimes in multiple formats:
/// - **`PostgreSQL` TIMESTAMP**: "YYYY-MM-DD HH:MM:SS.ffffff" (`CURRENT_TIMESTAMP` with fractional seconds)
/// - **RFC3339**: "2025-01-01T12:00:00+00:00" (used by programmatic inserts)
/// - **Unix timestamp**: Integer (seconds since epoch)
///
/// This helper handles all formats for consistent datetime parsing.
pub fn parse_database_datetime(value: &serde_json::Value) -> Option<DateTime<Utc>> {
    if let Some(s) = value.as_str() {
        // Try PostgreSQL TIMESTAMP format (with fractional seconds)
        // Format: "2025-01-01 12:00:00.123456"
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
            return Some(dt.and_utc());
        }

        // Try RFC3339 format (used by programmatic inserts)
        // Format: "2025-01-01T12:00:00+00:00"
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.with_timezone(&Utc));
        }

        None
    } else if let Some(ts) = value.as_i64() {
        DateTime::from_timestamp(ts, 0)
    } else {
        None
    }
}

/// Database value enum representing all possible `PostgreSQL` column types.
///
/// This enum provides a unified representation of `PostgreSQL` database values.
/// Type-specific NULL variants ensure proper `PostgreSQL` type coercion when binding parameters.
#[derive(Debug, Clone)]
pub enum DbValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Timestamp(DateTime<Utc>),
    StringArray(Vec<String>),
    NullString,
    NullInt,
    NullFloat,
    NullBool,
    NullBytes,
    NullTimestamp,
    NullStringArray,
}

/// Trait for converting Rust types to database-compatible values.
///
/// Implement this trait for custom types that need to be used as query parameters.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::{ToDbValue, DbValue};
///
/// struct UserId(String);
///
/// impl ToDbValue for UserId {
///     fn to_db_value(&self) -> DbValue {
///         DbValue::String(self.0.clone())
///     }
/// }
/// ```
pub trait ToDbValue: Send + Sync {
    fn to_db_value(&self) -> DbValue;

    /// Returns the appropriate NULL variant for this type.
    /// Used by Option<T> to return type-specific NULLs.
    fn null_db_value() -> DbValue
    where
        Self: Sized,
    {
        DbValue::NullString
    }
}

impl ToDbValue for &str {
    fn to_db_value(&self) -> DbValue {
        DbValue::String((*self).to_string())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullString
    }
}

impl ToDbValue for String {
    fn to_db_value(&self) -> DbValue {
        DbValue::String(self.clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullString
    }
}

impl ToDbValue for &String {
    fn to_db_value(&self) -> DbValue {
        DbValue::String((*self).clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullString
    }
}

impl ToDbValue for i32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(i64::from(*self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for i64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for u32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(i64::from(*self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for u64 {
    #[allow(clippy::cast_possible_wrap)]
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(*self as i64)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for f32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Float(f64::from(*self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullFloat
    }
}

impl ToDbValue for f64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Float(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullFloat
    }
}

impl ToDbValue for &f64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Float(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullFloat
    }
}

impl ToDbValue for &i32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(i64::from(**self))
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for &i64 {
    fn to_db_value(&self) -> DbValue {
        DbValue::Int(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullInt
    }
}

impl ToDbValue for &bool {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bool(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBool
    }
}

impl ToDbValue for bool {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bool(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBool
    }
}

impl ToDbValue for Vec<u8> {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bytes(self.clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBytes
    }
}

impl ToDbValue for &[u8] {
    fn to_db_value(&self) -> DbValue {
        DbValue::Bytes(self.to_vec())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullBytes
    }
}

impl<T: ToDbValue> ToDbValue for Option<T> {
    fn to_db_value(&self) -> DbValue {
        match self {
            Some(v) => v.to_db_value(),
            None => T::null_db_value(),
        }
    }
}

impl ToDbValue for DateTime<Utc> {
    fn to_db_value(&self) -> DbValue {
        DbValue::Timestamp(*self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullTimestamp
    }
}

impl ToDbValue for &DateTime<Utc> {
    fn to_db_value(&self) -> DbValue {
        DbValue::Timestamp(**self)
    }

    fn null_db_value() -> DbValue {
        DbValue::NullTimestamp
    }
}

impl ToDbValue for Vec<String> {
    fn to_db_value(&self) -> DbValue {
        DbValue::StringArray(self.clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullStringArray
    }
}

impl ToDbValue for &Vec<String> {
    fn to_db_value(&self) -> DbValue {
        DbValue::StringArray((*self).clone())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullStringArray
    }
}

impl ToDbValue for &[String] {
    fn to_db_value(&self) -> DbValue {
        DbValue::StringArray(self.to_vec())
    }

    fn null_db_value() -> DbValue {
        DbValue::NullStringArray
    }
}

/// Trait for converting database values to Rust types.
///
/// Implement this trait for custom types that need to be deserialized from query results.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::{FromDbValue, DbValue};
/// use anyhow::{anyhow, Result};
///
/// struct UserId(String);
///
/// impl FromDbValue for UserId {
///     fn from_db_value(value: &DbValue) -> Result<Self> {
///         match value {
///             DbValue::String(s) => Ok(UserId(s.clone())),
///             _ => Err(anyhow!("Invalid UserId type")),
///         }
///     }
/// }
/// ```
pub trait FromDbValue: Sized {
    fn from_db_value(value: &DbValue) -> Result<Self>;
}

impl FromDbValue for String {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::String(s) => Ok(s.clone()),
            DbValue::Int(i) => Ok(i.to_string()),
            DbValue::Float(f) => Ok(f.to_string()),
            DbValue::Bool(b) => Ok(b.to_string()),
            DbValue::Timestamp(dt) => Ok(dt.to_rfc3339()),
            DbValue::StringArray(arr) => {
                Ok(serde_json::to_string(arr).unwrap_or_else(|_| "[]".to_string()))
            },
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to String")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to String")),
        }
    }
}

impl FromDbValue for i64 {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Int(i) => Ok(*i),
            #[allow(clippy::cast_possible_truncation)]
            DbValue::Float(f) => Ok(*f as Self),
            DbValue::Bool(b) => Ok(Self::from(*b)),
            DbValue::String(s) => s.parse().map_err(|_| anyhow!("Cannot parse {s} as i64")),
            DbValue::StringArray(_) => Err(anyhow!("Cannot convert StringArray to i64")),
            DbValue::Timestamp(_) => Err(anyhow!("Cannot convert Timestamp to i64")),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to i64")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to i64")),
        }
    }
}

impl FromDbValue for i32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from_db_value(value: &DbValue) -> Result<Self> {
        i64::from_db_value(value).map(|v| v as Self)
    }
}

impl FromDbValue for u64 {
    #[allow(clippy::cast_sign_loss)]
    fn from_db_value(value: &DbValue) -> Result<Self> {
        i64::from_db_value(value).map(|v| v as Self)
    }
}

impl FromDbValue for u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn from_db_value(value: &DbValue) -> Result<Self> {
        i64::from_db_value(value).map(|v| v as Self)
    }
}

impl FromDbValue for f64 {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Float(f) => Ok(*f),
            #[allow(clippy::cast_precision_loss)]
            DbValue::Int(i) => Ok(*i as Self),
            DbValue::String(s) => s.parse().map_err(|_| anyhow!("Cannot parse {s} as f64")),
            DbValue::StringArray(_) => Err(anyhow!("Cannot convert StringArray to f64")),
            DbValue::Timestamp(_) => Err(anyhow!("Cannot convert Timestamp to f64")),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to f64")),
            DbValue::Bool(_) => Err(anyhow!("Cannot convert Bool to f64")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to f64")),
        }
    }
}

impl FromDbValue for bool {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Bool(b) => Ok(*b),
            DbValue::Int(i) => Ok(*i != 0),
            DbValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(true),
                "false" | "0" | "no" => Ok(false),
                _ => Err(anyhow!("Cannot parse {s} as bool")),
            },
            DbValue::StringArray(_) => Err(anyhow!("Cannot convert StringArray to bool")),
            DbValue::Timestamp(_) => Err(anyhow!("Cannot convert Timestamp to bool")),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to bool")),
            DbValue::Float(_) => Err(anyhow!("Cannot convert Float to bool")),
            DbValue::Bytes(_) => Err(anyhow!("Cannot convert Bytes to bool")),
        }
    }
}

impl FromDbValue for Vec<u8> {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::Bytes(b) => Ok(b.clone()),
            DbValue::String(s) => Ok(s.as_bytes().to_vec()),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to Vec<u8>")),
            _ => Err(anyhow!("Cannot convert {value:?} to Vec<u8>")),
        }
    }
}

impl<T: FromDbValue> FromDbValue for Option<T> {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Ok(None),
            _ => T::from_db_value(value).map(Some),
        }
    }
}

impl FromDbValue for DateTime<Utc> {
    fn from_db_value(value: &DbValue) -> Result<Self> {
        match value {
            DbValue::String(s) => parse_database_datetime(&serde_json::Value::String(s.clone()))
                .ok_or_else(|| anyhow!("Cannot parse {s} as DateTime<Utc>")),
            DbValue::Timestamp(dt) => Ok(*dt),
            DbValue::Int(ts) => Self::from_timestamp(*ts, 0)
                .ok_or_else(|| anyhow!("Invalid Unix timestamp: {ts}")),
            DbValue::NullString
            | DbValue::NullInt
            | DbValue::NullFloat
            | DbValue::NullBool
            | DbValue::NullBytes
            | DbValue::NullTimestamp
            | DbValue::NullStringArray => Err(anyhow!("Cannot convert NULL to DateTime<Utc>")),
            _ => Err(anyhow!("Cannot convert {value:?} to DateTime<Utc>")),
        }
    }
}

/// `PostgreSQL` database query.
///
/// This struct holds a `PostgreSQL` query that can be executed through the [`DatabaseProvider`].
/// Stores a `PostgreSQL` query that is loaded at compile time.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::DatabaseQuery;
///
/// const CREATE_USER: DatabaseQuery = DatabaseQuery::new(
///     include_str!("queries/postgres/create_user.sql"),
/// );
///
/// db.execute(&CREATE_USER, &[&"Alice"]).await?;
/// ```
///
/// Use the `database_query!` macro for cleaner syntax:
///
/// ```rust
/// const CREATE_USER: DatabaseQuery = database_query!("users/create");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DatabaseQuery {
    postgres: &'static str,
}

impl DatabaseQuery {
    pub const fn new(query: &'static str) -> Self {
        Self { postgres: query }
    }

    /// Get the `PostgreSQL` query string (only variant supported)
    pub const fn postgres(&self) -> &str {
        self.postgres
    }

    /// Deprecated: Use `postgres()` instead. Kept for backward compatibility.
    #[deprecated(
        since = "0.1.0",
        note = "Use postgres() instead - SQLite is no longer supported"
    )]
    pub const fn select(&self, _is_postgres: bool) -> &str {
        self.postgres
    }

    /// Deprecated: Use `postgres()` instead. Kept for backward compatibility.
    #[deprecated(
        since = "0.1.0",
        note = "Use postgres() instead - SQLite is no longer supported"
    )]
    pub fn get(&self, _db: &dyn crate::DatabaseProvider) -> &str {
        self.postgres
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseQueryEnum {
    SaveAgentCapabilities,
    GetAgentCapabilities,
    RegisterAgent,
    MarkAgentStopped,
    MarkAgentCrashed,
    MarkAgentError,
    RemoveAgentService,
    ListRunningAgents,
    GetAgentStatus,
    UpdateAgentHealth,
    InsertArtifact,
    InsertArtifactPart,
    GetArtifactById,
    GetArtifactsByContext,
    GetArtifactsByUser,
    GetArtifactsByUserLimited,
    GetArtifactsByTask,
    GetArtifactParts,
    LinkArtifactToContext,
    TrackAgentInContext,
    InsertContext,
    GetContext,
    GetContextsByUser,
    UpdateContext,
    DeleteContext,
    SearchContexts,
    GetLastAgentForContext,
    InsertMessage,
    InsertMessagePartText,
    InsertMessagePartFile,
    InsertMessagePartData,
    GetMessageParts,
    GetMessagePartsByTask,
    GetTaskMessages,
    GetContextMessages,
    InsertTask,
    InsertTaskSimple,
    GetTask,
    ListTasksByContext,
    GetTasksByUser,
    GetTasksByUserPaged,
    UpdateTaskStatus,
    UpdateTaskStatusCompleted,
    UpdateTaskState,
    UpdateTaskWithMetadata,
    GetTaskSummary,
    GetMaxSequenceNumber,
    DeleteMessageParts,
    DeleteTaskMessage,
    GetNewToolExecutionsSince,
    GetNewTaskStatusChangesSince,
    GetNewArtifactsSince,
    GetContextUpdatesSince,
    InsertPushNotificationConfig,
    GetPushNotificationConfigById,
    DeletePushNotificationConfig,
    DeletePushNotificationConfigById,
    GetUserByContext,
    InsertContextNotification,
    MarkNotificationBroadcasted,
    GetMessageById,
    GetTaskContextUser,
    InsertClient,
    InsertClientBase,
    GetClientByClientId,
    UpdateClient,
    DeleteClient,
    ActivateClient,
    DeactivateClient,
    ListClients,
    CountClients,
    InsertRedirectUri,
    LoadRedirectUris,
    DeleteRedirectUris,
    InsertGrantType,
    LoadGrantTypes,
    DeleteGrantTypes,
    InsertResponseType,
    LoadResponseTypes,
    DeleteResponseTypes,
    InsertScope,
    LoadScopes,
    DeleteScopes,
    InsertContact,
    LoadContacts,
    DeleteContacts,
    DeleteUnusedClients,
    ListUnusedClients,
    ListStaleClients,
    DeactivateOldTestClients,
    UpdateClientLastUsed,
    GetClientAnalytics,
    GetClientAnalyticsById,
    GetClientErrors,
    GetClientErrorsById,
    InsertAuthorizationCode,
    GetAuthorizationCode,
    RevokeAuthorizationCode,
    InsertAccessToken,
    GetAccessToken,
    RevokeAccessToken,
    InsertRefreshToken,
    GetRefreshToken,
    RevokeRefreshToken,
    CleanupExpiredTokens,
    CreateSession,
    GetSession,
    UpdateSessionLastActivity,
    EndSession,
    DeleteExpiredSessions,
    CountRecentSessions,
    GetAuthenticatedUser,
    InsertCredential,
    GetCredential,
    GetCredentialsByUserId,
    UpdateCredentialCounter,
    DeleteCredential,
    DeleteInactiveClients,
    DeleteOldTestClients,
    ListInactiveClients,
    ListOldClients,
    DeleteStaleClients,
    UpdateClientSecret,
    UpdateLastUsed,
    GetAuthorizationCodeClientId,
    DeleteRefreshToken,
    MarkAuthorizationCodeUsed,
    DeleteExpiredRefreshTokens,
    GetRoles,
    CheckRoleExists,
    GetDefaultRoles,
    GetOAuthUser,
    GetPlatformOverview,
    GetTopUsers,
    GetTopAgents,
    GetTopTools,
    GetActivityTrend,
    GetUserMetrics,
    GetCostBreakdown,
    GetSystemHealth,
    CreateAnalyticsSession,
    GetAnalyticsSession,
    SessionExists,
    EndAnalyticsSession,
    MarkSessionAsScanner,
    UpdateSessionQuality,
    UpdateSessionActivity,
    UpdateSessionEndpoints,
    RecordEndpointRequest,
    IncrementSessionAiUsage,
    IncrementSessionTaskActivity,
    GetAnalyticsActiveSessions,
    CleanupInactiveAnalyticsSessions,
    FindSessionByFingerprint,
    FindSessionByFingerprintAny,
    GetEndpointRequestsBySession,
    CleanupExpiredAnonymousSessions,
    MigrateSessionToUser,
    UpdateSessionContexts,
    UpdateSessionUserAgentTasks,
    UpdateSessionUserTaskMessages,
    UpdateSessionUserAiRequests,
    UpdateSessionUserLogs,
    UpdateSessionUserToolExecutions,
    DeleteTemporarySessionUser,
    DeleteContextBySession,
    DeleteSessionById,
    RecordEvent,
    GetEventsBySession,
    GetEventsSummary,
    GetDailyActivityTrendBase,
    GetSystemHealthMetrics,
    GetAgentUsageAnalyticsBase,
    GetTopUsersSummary,
    GetSessionQualityMetrics,
    GetUserAnalyticsSummary,
    CreateUser,
    GetUserById,
    GetUserByName,
    GetUserByEmail,
    DeleteUser,
    ListUsers,
    SearchUsers,
    CountUsers,
    AssignRole,
    RemoveRole,
    GetUserRoles,
    FindByRole,
    FindFirstAdmin,
    FindFirstUser,
    CreateUserSession,
    GetUserSession,
    DeleteUserSession,
    ListUserSessions,
    CreateAnonymousUser,
    GetAnonymousUser,
    ConvertAnonymousUser,
    DeleteAnonymousUser,
    IsTemporaryAnonymous,
    CleanupOldAnonymousUsers,
    UpdateUserEmail,
    UpdateUserEmailFullName,
    UpdateUserEmailStatus,
    UpdateUserFullName,
    UpdateUserFullNameStatus,
    UpdateUserStatus,
    UpdateUserAllFields,
    CreateContent,
    GetContentById,
    GetContentByUrl,
    GetContentBySourceAndSlug,
    GetSocialContentByParent,
    GetContentByVersionHash,
    ListContent,
    ListAllContent,
    ListContentBySource,
    SearchByCategory,
    SearchByTags,
    SearchContentByKeyword,
    UpdateContent,
    UpdateContentImage,
    DeleteContent,
    DeleteContentBySource,
    AddLinksColumnToContent,
    CreateTag,
    GetTagById,
    GetTagByName,
    ListTags,
    DeleteTag,
    LinkTagToContent,
    UnlinkTagFromContent,
    UnlinkAllTagsFromContent,
    GetTagsByContent,
    CreateLink,
    GetLinkById,
    GetLinkByShortCode,
    ListLinksByCampaign,
    ListLinksBySourceContent,
    IncrementLinkClicks,
    RecordClick,
    GetClicksByLink,
    CreateSkill,
    GetSkillById,
    GetSkillByFilePath,
    ListEnabledSkills,
    ListAllSkills,
    UpdateSkill,
    CheckSessionClickedLink,
    GetLinkPerformance,
    GetAggregatedLinkPerformance,
    GetCampaignPerformance,
    GetContentJourneyMap,
    CreateCategory,
    GetCategoryById,
    GetCategoryByName,
    ListCategories,
    DeleteCategory,
    InsertToolExecution,
    GetToolExecution,
    ListToolExecutionsBySession,
    ListToolExecutionsByUser,
    UpdateToolExecutionStatus,
    UpdateToolExecutionResult,
    RegisterTool,
    GetToolMetadata,
    ListAvailableTools,
    UpdateToolMetadata,
    GetToolUsageStats,
    GetToolErrorRate,
    GetToolPerformanceMetrics,
    RegisterMcpServer,
    ListMcpServers,
    UpdateMcpServerStatus,
    RemoveMcpServer,
    LinkToolToServer,
    GetToolsByServer,
    UnlinkToolFromServer,
    GetMcpConfig,
    UpdateMcpConfig,
    InsertAiRequest,
    InsertAiImageRequest,
    GetAiRequest,
    ListAiRequestsBySession,
    ListAiRequestsByUser,
    UpdateAiRequestStatus,
    InsertRequestMessage,
    InsertResponseMessage,
    GetAiMessageMaxSequence,
    InsertGeneratedImage,
    GetGeneratedImageByUuid,
    ListGeneratedImagesByUser,
    DeleteGeneratedImage,
    GetRequestMessages,
    InsertToolCall,
    GetToolCalls,
    RegisterProvider,
    GetProvider,
    ListProviders,
    UpdateProviderConfig,
    RegisterModel,
    GetModel,
    ListModelsByProvider,
    UpdateModelCapabilities,
    GetTokenUsageByModel,
    GetUserAiUsageAll,
    GetUserAiUsageWithDateRange,
    GetUserAiUsageSinceDate,
    GetUserAiUsageUntilDate,
    GetProviderUsageAll,
    GetProviderUsageByUser,
    CreateLog,
    GetLog,
    ListLogs,
    ListLogsPaginated,
    DeleteLog,
    DeleteOldLogs,
    LogAnalyticsEvent,
    GetLogsByLevel,
    GetLogsByModule,
    GetLogsByUser,
    GetLogsBySession,
    GetLogStats,
    GetErrorRate,
    VacuumLogs,
    OptimizeLogIndices,
    GetLogRetentionMetrics,
    ArchiveOldLogs,
    CheckConfigTableExists,
    InsertModule,
    GetAllModules,
    EnableModule,
    DisableModule,
    DeleteModule,
    UpdateModule,
    CreateVariable,
    GetVariable,
    GetVariableById,
    ListVariables,
    ListVariablesByCategory,
    DeleteVariable,
    UpdateVariable,
    ListActiveUserSessions,
    ListRecentUserSessions,
    GetUserActivity,
    UpdateUserRoles,
    GetAgentConversationStats,
    GetTopAgentsByConversations,
    GetTrafficSummary,
    GetDeviceBreakdown,
    GetGeoBreakdown,
    GetClientBreakdown,
    GetVisitorJourney,
    GetTrafficSources,
    GetUtmCampaigns,
    GetLandingPages,
    GetBotScannerSummary,
    GetScannerPaths,
    GetTrafficTrendHourly,
    GetTrafficTrendDaily,
    GetConversationSummary,
    GetConversationsByAgent,
    GetConversationsByStatus,
    GetRecentConversations,
    GetRecentConversationsPaginated,
    GetConversationTrends,
    GetConversationMetricsMultiPeriod,
    GetTopSubjects,
    GetSubjectTrends,
    AnalyzeConversation,
    GetTopContent,
    GetCategoryPerformance,
    GetContentTrends,
    GetDailyViewsPerContent,
    GetTopReferrers,
    GetDeviceLocation,
    UpsertScheduledJob,
    GetScheduledJob,
    ListEnabledJobs,
    UpdateJobExecution,
    IncrementJobRunCount,
    CreateEvaluation,
    GetEvaluationByContext,
    GetEvaluationMetrics,
    GetLowScoringConversations,
    GetEvaluationQualityDistribution,
    GetRecentEvaluations,
    GetConversationsByLocation,
    GetUnevaluatedConversations,
    GetTopIssuesEncountered,
    GetGoalAchievementStats,
    GetDetailedEvaluations,
    FetchTraceEvents,
    CliListTables,
    CliDescribeTable,
    CliGetTableCount,
    CliGetDbVersion,
    DeleteOrphanedLogs,
    DeleteOrphanedAnalyticsEvents,
    DeleteOrphanedMcpExecutions,
    DeleteExpiredOAuthCodes,
    DeleteExpiredOAuthTokens,
    GetContentClickMetrics,
    GetSessionClickEngagement,
}

impl DatabaseQueryEnum {
    #[allow(clippy::match_same_arms)]
    pub const fn module(self) -> &'static str {
        match self {
            Self::SaveAgentCapabilities
            | Self::GetAgentCapabilities
            | Self::RegisterAgent
            | Self::MarkAgentStopped
            | Self::MarkAgentCrashed
            | Self::MarkAgentError
            | Self::RemoveAgentService
            | Self::ListRunningAgents
            | Self::GetAgentStatus
            | Self::UpdateAgentHealth
            | Self::InsertArtifact
            | Self::InsertArtifactPart
            | Self::GetArtifactById
            | Self::GetArtifactsByContext
            | Self::GetArtifactsByUser
            | Self::GetArtifactsByUserLimited
            | Self::GetArtifactsByTask
            | Self::GetArtifactParts
            | Self::LinkArtifactToContext
            | Self::TrackAgentInContext
            | Self::InsertContext
            | Self::GetContext
            | Self::GetContextsByUser
            | Self::UpdateContext
            | Self::DeleteContext
            | Self::SearchContexts
            | Self::GetLastAgentForContext
            | Self::InsertMessage
            | Self::InsertMessagePartText
            | Self::InsertMessagePartFile
            | Self::InsertMessagePartData
            | Self::GetMessageParts
            | Self::GetMessagePartsByTask
            | Self::GetTaskMessages
            | Self::GetContextMessages
            | Self::InsertTask
            | Self::InsertTaskSimple
            | Self::GetTask
            | Self::ListTasksByContext
            | Self::GetTasksByUser
            | Self::GetTasksByUserPaged
            | Self::UpdateTaskStatus
            | Self::UpdateTaskStatusCompleted
            | Self::UpdateTaskState
            | Self::UpdateTaskWithMetadata
            | Self::GetTaskSummary
            | Self::GetMaxSequenceNumber
            | Self::DeleteMessageParts
            | Self::DeleteTaskMessage
            | Self::GetNewToolExecutionsSince
            | Self::GetNewTaskStatusChangesSince
            | Self::GetNewArtifactsSince
            | Self::GetContextUpdatesSince
            | Self::InsertPushNotificationConfig
            | Self::GetPushNotificationConfigById
            | Self::DeletePushNotificationConfig
            | Self::DeletePushNotificationConfigById
            | Self::GetUserByContext
            | Self::InsertContextNotification
            | Self::MarkNotificationBroadcasted
            | Self::GetMessageById
            | Self::GetTaskContextUser
            | Self::CreateSkill
            | Self::GetSkillById
            | Self::GetSkillByFilePath
            | Self::ListEnabledSkills
            | Self::ListAllSkills
            | Self::UpdateSkill => "agent",

            Self::InsertClient
            | Self::InsertClientBase
            | Self::GetClientByClientId
            | Self::UpdateClient
            | Self::DeleteClient
            | Self::ActivateClient
            | Self::DeactivateClient
            | Self::ListClients
            | Self::CountClients
            | Self::InsertRedirectUri
            | Self::LoadRedirectUris
            | Self::DeleteRedirectUris
            | Self::InsertGrantType
            | Self::LoadGrantTypes
            | Self::DeleteGrantTypes
            | Self::InsertResponseType
            | Self::LoadResponseTypes
            | Self::DeleteResponseTypes
            | Self::InsertScope
            | Self::LoadScopes
            | Self::DeleteScopes
            | Self::InsertContact
            | Self::LoadContacts
            | Self::DeleteContacts
            | Self::DeleteUnusedClients
            | Self::ListUnusedClients
            | Self::ListStaleClients
            | Self::DeactivateOldTestClients
            | Self::UpdateClientLastUsed
            | Self::GetClientAnalytics
            | Self::GetClientAnalyticsById
            | Self::GetClientErrors
            | Self::GetClientErrorsById
            | Self::InsertAuthorizationCode
            | Self::GetAuthorizationCode
            | Self::RevokeAuthorizationCode
            | Self::InsertAccessToken
            | Self::GetAccessToken
            | Self::RevokeAccessToken
            | Self::InsertRefreshToken
            | Self::GetRefreshToken
            | Self::RevokeRefreshToken
            | Self::CleanupExpiredTokens
            | Self::CreateSession
            | Self::GetSession
            | Self::UpdateSessionLastActivity
            | Self::EndSession
            | Self::DeleteExpiredSessions
            | Self::CountRecentSessions
            | Self::GetAuthenticatedUser
            | Self::InsertCredential
            | Self::GetCredential
            | Self::GetCredentialsByUserId
            | Self::UpdateCredentialCounter
            | Self::DeleteCredential
            | Self::DeleteInactiveClients
            | Self::DeleteOldTestClients
            | Self::ListInactiveClients
            | Self::ListOldClients
            | Self::DeleteStaleClients
            | Self::UpdateClientSecret
            | Self::UpdateLastUsed
            | Self::GetAuthorizationCodeClientId
            | Self::DeleteRefreshToken
            | Self::MarkAuthorizationCodeUsed
            | Self::DeleteExpiredRefreshTokens
            | Self::GetRoles
            | Self::CheckRoleExists
            | Self::GetDefaultRoles
            | Self::GetOAuthUser => "oauth",

            Self::GetPlatformOverview
            | Self::GetTopUsers
            | Self::GetTopAgents
            | Self::GetTopTools
            | Self::GetActivityTrend
            | Self::GetUserMetrics
            | Self::GetCostBreakdown
            | Self::GetSystemHealth
            | Self::CreateAnalyticsSession
            | Self::GetAnalyticsSession
            | Self::SessionExists
            | Self::EndAnalyticsSession
            | Self::MarkSessionAsScanner
            | Self::UpdateSessionQuality
            | Self::UpdateSessionActivity
            | Self::UpdateSessionEndpoints
            | Self::RecordEndpointRequest
            | Self::IncrementSessionAiUsage
            | Self::IncrementSessionTaskActivity
            | Self::GetAnalyticsActiveSessions
            | Self::CleanupInactiveAnalyticsSessions
            | Self::FindSessionByFingerprint
            | Self::FindSessionByFingerprintAny
            | Self::GetEndpointRequestsBySession
            | Self::CleanupExpiredAnonymousSessions
            | Self::MigrateSessionToUser
            | Self::UpdateSessionContexts
            | Self::UpdateSessionUserAgentTasks
            | Self::UpdateSessionUserTaskMessages
            | Self::UpdateSessionUserAiRequests
            | Self::UpdateSessionUserLogs
            | Self::UpdateSessionUserToolExecutions
            | Self::DeleteTemporarySessionUser
            | Self::DeleteContextBySession
            | Self::DeleteSessionById
            | Self::RecordEvent
            | Self::GetEventsBySession
            | Self::GetEventsSummary
            | Self::GetDailyActivityTrendBase
            | Self::GetSystemHealthMetrics
            | Self::GetAgentUsageAnalyticsBase
            | Self::GetTopUsersSummary
            | Self::GetSessionQualityMetrics
            | Self::GetUserAnalyticsSummary => "core",

            Self::CreateUser
            | Self::GetUserById
            | Self::GetUserByName
            | Self::GetUserByEmail
            | Self::DeleteUser
            | Self::ListUsers
            | Self::SearchUsers
            | Self::CountUsers
            | Self::AssignRole
            | Self::RemoveRole
            | Self::GetUserRoles
            | Self::FindByRole
            | Self::FindFirstAdmin
            | Self::FindFirstUser
            | Self::CreateUserSession
            | Self::GetUserSession
            | Self::DeleteUserSession
            | Self::ListUserSessions
            | Self::CreateAnonymousUser
            | Self::GetAnonymousUser
            | Self::ConvertAnonymousUser
            | Self::DeleteAnonymousUser
            | Self::IsTemporaryAnonymous
            | Self::CleanupOldAnonymousUsers
            | Self::UpdateUserEmail
            | Self::UpdateUserEmailFullName
            | Self::UpdateUserEmailStatus
            | Self::UpdateUserFullName
            | Self::UpdateUserFullNameStatus
            | Self::UpdateUserStatus
            | Self::UpdateUserAllFields => "users",

            Self::CreateContent
            | Self::GetContentById
            | Self::GetContentByUrl
            | Self::GetContentBySourceAndSlug
            | Self::GetSocialContentByParent
            | Self::GetContentByVersionHash
            | Self::ListContent
            | Self::ListAllContent
            | Self::SearchByCategory
            | Self::SearchByTags
            | Self::SearchContentByKeyword
            | Self::UpdateContent
            | Self::UpdateContentImage
            | Self::DeleteContent
            | Self::DeleteContentBySource
            | Self::AddLinksColumnToContent
            | Self::CreateTag
            | Self::GetTagById
            | Self::GetTagByName
            | Self::ListTags
            | Self::DeleteTag
            | Self::LinkTagToContent
            | Self::UnlinkTagFromContent
            | Self::UnlinkAllTagsFromContent
            | Self::GetTagsByContent
            | Self::CreateCategory
            | Self::GetCategoryById
            | Self::GetCategoryByName
            | Self::ListCategories
            | Self::DeleteCategory => "rag",

            Self::InsertToolExecution
            | Self::GetToolExecution
            | Self::ListToolExecutionsBySession
            | Self::ListToolExecutionsByUser
            | Self::UpdateToolExecutionStatus
            | Self::UpdateToolExecutionResult
            | Self::RegisterTool
            | Self::GetToolMetadata
            | Self::ListAvailableTools
            | Self::UpdateToolMetadata
            | Self::GetToolUsageStats
            | Self::GetToolErrorRate
            | Self::GetToolPerformanceMetrics
            | Self::RegisterMcpServer
            | Self::ListMcpServers
            | Self::UpdateMcpServerStatus
            | Self::RemoveMcpServer
            | Self::LinkToolToServer
            | Self::GetToolsByServer
            | Self::UnlinkToolFromServer
            | Self::GetMcpConfig
            | Self::UpdateMcpConfig => "mcp",

            Self::InsertAiRequest
            | Self::InsertAiImageRequest
            | Self::GetAiRequest
            | Self::ListAiRequestsBySession
            | Self::ListAiRequestsByUser
            | Self::UpdateAiRequestStatus
            | Self::InsertRequestMessage
            | Self::InsertResponseMessage
            | Self::GetAiMessageMaxSequence
            | Self::InsertGeneratedImage
            | Self::GetGeneratedImageByUuid
            | Self::ListGeneratedImagesByUser
            | Self::DeleteGeneratedImage
            | Self::GetRequestMessages
            | Self::InsertToolCall
            | Self::GetToolCalls
            | Self::RegisterProvider
            | Self::GetProvider
            | Self::ListProviders
            | Self::UpdateProviderConfig
            | Self::RegisterModel
            | Self::GetModel
            | Self::ListModelsByProvider
            | Self::UpdateModelCapabilities
            | Self::GetTokenUsageByModel
            | Self::GetUserAiUsageAll
            | Self::GetUserAiUsageWithDateRange
            | Self::GetUserAiUsageSinceDate
            | Self::GetUserAiUsageUntilDate
            | Self::GetProviderUsageAll
            | Self::GetProviderUsageByUser => "ai",

            Self::CreateLog
            | Self::GetLog
            | Self::ListLogs
            | Self::ListLogsPaginated
            | Self::DeleteLog
            | Self::DeleteOldLogs
            | Self::LogAnalyticsEvent
            | Self::GetLogsByLevel
            | Self::GetLogsByModule
            | Self::GetLogsByUser
            | Self::GetLogsBySession
            | Self::GetLogStats
            | Self::GetErrorRate
            | Self::VacuumLogs
            | Self::OptimizeLogIndices
            | Self::GetLogRetentionMetrics
            | Self::ArchiveOldLogs => "log",

            Self::CheckConfigTableExists
            | Self::InsertModule
            | Self::GetAllModules
            | Self::EnableModule
            | Self::DisableModule
            | Self::DeleteModule
            | Self::UpdateModule
            | Self::CreateVariable
            | Self::GetVariable
            | Self::GetVariableById
            | Self::ListVariables
            | Self::ListVariablesByCategory
            | Self::DeleteVariable
            | Self::UpdateVariable => "config",

            Self::ListActiveUserSessions
            | Self::ListRecentUserSessions
            | Self::GetUserActivity
            | Self::UpdateUserRoles => "users",

            Self::GetAgentConversationStats | Self::GetTopAgentsByConversations => "agent",

            Self::GetTrafficSummary
            | Self::GetDeviceBreakdown
            | Self::GetGeoBreakdown
            | Self::GetClientBreakdown
            | Self::GetVisitorJourney
            | Self::GetTrafficSources
            | Self::GetUtmCampaigns
            | Self::GetLandingPages
            | Self::GetBotScannerSummary
            | Self::GetScannerPaths
            | Self::GetTrafficTrendHourly
            | Self::GetTrafficTrendDaily
            | Self::GetConversationSummary
            | Self::GetConversationsByAgent
            | Self::GetConversationsByStatus
            | Self::GetRecentConversations
            | Self::GetRecentConversationsPaginated
            | Self::GetConversationTrends
            | Self::GetConversationMetricsMultiPeriod
            | Self::GetTopSubjects
            | Self::GetSubjectTrends
            | Self::AnalyzeConversation => "core",

            Self::GetTopContent
            | Self::GetCategoryPerformance
            | Self::GetContentTrends
            | Self::GetDailyViewsPerContent
            | Self::GetTopReferrers
            | Self::GetDeviceLocation
            | Self::ListContentBySource
            | Self::GetContentClickMetrics
            | Self::GetSessionClickEngagement
            | Self::GetLinkPerformance
            | Self::GetAggregatedLinkPerformance => "blog",

            Self::UpsertScheduledJob
            | Self::GetScheduledJob
            | Self::ListEnabledJobs
            | Self::UpdateJobExecution
            | Self::IncrementJobRunCount
            | Self::CreateEvaluation
            | Self::GetEvaluationByContext
            | Self::GetEvaluationMetrics
            | Self::GetLowScoringConversations
            | Self::GetEvaluationQualityDistribution
            | Self::GetRecentEvaluations
            | Self::GetConversationsByLocation
            | Self::GetUnevaluatedConversations
            | Self::GetTopIssuesEncountered
            | Self::GetGoalAchievementStats
            | Self::GetDetailedEvaluations => "scheduler",

            Self::FetchTraceEvents
            | Self::CliListTables
            | Self::CliDescribeTable
            | Self::CliGetTableCount
            | Self::CliGetDbVersion
            | Self::DeleteOrphanedLogs
            | Self::DeleteOrphanedAnalyticsEvents
            | Self::DeleteOrphanedMcpExecutions
            | Self::DeleteExpiredOAuthCodes
            | Self::DeleteExpiredOAuthTokens
            | Self::CreateLink
            | Self::GetLinkById
            | Self::GetLinkByShortCode
            | Self::ListLinksByCampaign
            | Self::ListLinksBySourceContent
            | Self::IncrementLinkClicks
            | Self::RecordClick
            | Self::GetClicksByLink
            | Self::CheckSessionClickedLink
            | Self::GetCampaignPerformance
            | Self::GetContentJourneyMap => "core",
        }
    }

    pub fn get(self, _db: &dyn crate::DatabaseProvider) -> &'static str {
        super::queries::agent_get_query(self)
            .or_else(|| super::queries::oauth_get_query(self))
            .or_else(|| super::queries::core_get_query(self))
            .or_else(|| super::queries::users_get_query(self))
            .or_else(|| super::queries::blog_get_query(self))
            .or_else(|| super::queries::mcp_get_query(self))
            .or_else(|| super::queries::ai_get_query(self))
            .or_else(|| super::queries::log_get_query(self))
            .or_else(|| super::queries::config_get_query(self))
            .or_else(|| super::queries::scheduler_get_query(self))
            .or_else(|| super::queries::content_manager_get_query(self))
            .unwrap_or_else(|| panic!("Query {self:?} not implemented"))
    }

    #[allow(clippy::match_same_arms)]
    pub const fn description(self) -> &'static str {
        match self {
            Self::SaveAgentCapabilities => "Save agent capabilities",
            Self::GetAgentCapabilities => "Get agent capabilities",
            Self::RegisterAgent => "Register agent service",
            Self::MarkAgentStopped => "Mark agent as stopped",
            Self::MarkAgentCrashed => "Mark agent as crashed",
            Self::MarkAgentError => "Mark agent with error",
            Self::RemoveAgentService => "Remove agent service",
            Self::ListRunningAgents => "List all running agents",
            Self::GetAgentStatus => "Get agent status",
            Self::UpdateAgentHealth => "Update agent health",
            Self::InsertArtifact => "Insert artifact",
            Self::InsertArtifactPart => "Insert artifact part",
            Self::GetArtifactById => "Get artifact by ID",
            Self::GetArtifactsByContext => "Get artifacts by context",
            Self::GetArtifactsByUser => "Get artifacts by user",
            Self::GetArtifactsByUserLimited => "Get artifacts by user with limit",
            Self::GetArtifactsByTask => "Get artifacts by task",
            Self::GetArtifactParts => "Get artifact parts",
            Self::LinkArtifactToContext => "Link artifact to context",
            Self::TrackAgentInContext => "Track agent in context",
            Self::InsertContext => "Insert context",
            Self::GetContext => "Get context",
            Self::GetContextsByUser => "Get contexts by user",
            Self::UpdateContext => "Update context",
            Self::DeleteContext => "Delete context",
            Self::SearchContexts => "Search contexts",
            Self::GetLastAgentForContext => "Get last agent used in context",
            Self::InsertMessage => "Insert message",
            Self::InsertMessagePartText => "Insert text message part",
            Self::InsertMessagePartFile => "Insert file message part",
            Self::InsertMessagePartData => "Insert data message part",
            Self::GetMessageParts => "Get message parts",
            Self::GetMessagePartsByTask => "Get message parts by task",
            Self::GetTaskMessages => "Get task messages",
            Self::GetContextMessages => "Get context messages",
            Self::InsertTask => "Insert task",
            Self::InsertTaskSimple => "Insert task (simple)",
            Self::GetTask => "Get task",
            Self::ListTasksByContext => "List tasks by context",
            Self::GetTasksByUser => "Get tasks by user",
            Self::GetTasksByUserPaged => "Get tasks by user with pagination",
            Self::UpdateTaskStatus => "Update task status",
            Self::UpdateTaskStatusCompleted => "Update task status to completed",
            Self::UpdateTaskState => "Update task state",
            Self::UpdateTaskWithMetadata => "Update task with metadata",
            Self::GetTaskSummary => "Get task summary",
            Self::GetMaxSequenceNumber => "Get max sequence number",
            Self::DeleteMessageParts => "Delete message parts",
            Self::DeleteTaskMessage => "Delete task message",
            Self::GetNewToolExecutionsSince => "Get new tool executions since timestamp",
            Self::GetNewTaskStatusChangesSince => "Get new task status changes since timestamp",
            Self::GetNewArtifactsSince => "Get new artifacts since timestamp",
            Self::GetContextUpdatesSince => "Get context updates since timestamp",
            Self::InsertPushNotificationConfig => "Insert push notification config",
            Self::GetPushNotificationConfigById => "Get push notification config by ID",
            Self::DeletePushNotificationConfig => "Delete push notification config",
            Self::DeletePushNotificationConfigById => "Delete push notification config by ID",
            Self::GetUserByContext => "Get user ID by context ID",
            Self::InsertContextNotification => "Insert context notification",
            Self::MarkNotificationBroadcasted => "Mark notification as broadcasted",
            Self::GetMessageById => "Get AI request message by ID",
            Self::GetTaskContextUser => "Get context and user from task",
            Self::CreateSkill => "Create skill",
            Self::GetSkillById => "Get skill by ID",
            Self::GetSkillByFilePath => "Get skill by file path",
            Self::ListEnabledSkills => "List enabled skills",
            Self::ListAllSkills => "List all skills",
            Self::UpdateSkill => "Update skill",
            Self::InsertClient => "Insert OAuth client",
            Self::InsertClientBase => "Insert OAuth client (base)",
            Self::GetClientByClientId => "Get client by client ID",
            Self::UpdateClient => "Update client",
            Self::DeleteClient => "Delete client",
            Self::ActivateClient => "Activate client",
            Self::DeactivateClient => "Deactivate client",
            Self::ListClients => "List clients",
            Self::CountClients => "Count clients",
            Self::InsertRedirectUri => "Insert redirect URI",
            Self::LoadRedirectUris => "Load redirect URIs",
            Self::DeleteRedirectUris => "Delete redirect URIs",
            Self::InsertGrantType => "Insert grant type",
            Self::LoadGrantTypes => "Load grant types",
            Self::DeleteGrantTypes => "Delete grant types",
            Self::InsertResponseType => "Insert response type",
            Self::LoadResponseTypes => "Load response types",
            Self::DeleteResponseTypes => "Delete response types",
            Self::InsertScope => "Insert scope",
            Self::LoadScopes => "Load scopes",
            Self::DeleteScopes => "Delete scopes",
            Self::InsertContact => "Insert contact",
            Self::LoadContacts => "Load contacts",
            Self::DeleteContacts => "Delete contacts",
            Self::DeleteUnusedClients => "Delete unused clients",
            Self::ListUnusedClients => "List unused clients",
            Self::ListStaleClients => "List stale clients",
            Self::DeactivateOldTestClients => "Deactivate old test clients",
            Self::UpdateClientLastUsed => "Update client last used",
            Self::GetClientAnalytics => "Get client analytics",
            Self::GetClientAnalyticsById => "Get client analytics by ID",
            Self::GetClientErrors => "Get client errors",
            Self::GetClientErrorsById => "Get client errors by ID",
            Self::InsertAuthorizationCode => "Insert authorization code",
            Self::GetAuthorizationCode => "Get authorization code",
            Self::RevokeAuthorizationCode => "Revoke authorization code",
            Self::InsertAccessToken => "Insert access token",
            Self::GetAccessToken => "Get access token",
            Self::RevokeAccessToken => "Revoke access token",
            Self::InsertRefreshToken => "Insert refresh token",
            Self::GetRefreshToken => "Get refresh token",
            Self::RevokeRefreshToken => "Revoke refresh token",
            Self::CleanupExpiredTokens => "Cleanup expired tokens",
            Self::CreateSession => "Create OAuth session",
            Self::GetSession => "Get session",
            Self::UpdateSessionLastActivity => "Update session last activity",
            Self::EndSession => "End session",
            Self::DeleteExpiredSessions => "Delete expired sessions",
            Self::CountRecentSessions => "Count recent sessions",
            Self::GetAuthenticatedUser => "Get authenticated user",
            Self::InsertCredential => "Insert WebAuthn credential",
            Self::GetCredential => "Get credential",
            Self::GetCredentialsByUserId => "Get credentials by user ID",
            Self::UpdateCredentialCounter => "Update credential counter",
            Self::DeleteCredential => "Delete credential",
            Self::DeleteInactiveClients => "Delete inactive OAuth clients",
            Self::DeleteOldTestClients => "Delete old test OAuth clients",
            Self::ListInactiveClients => "List inactive OAuth clients",
            Self::ListOldClients => "List old OAuth clients",
            Self::DeleteStaleClients => "Delete stale OAuth clients",
            Self::UpdateClientSecret => "Update OAuth client secret",
            Self::UpdateLastUsed => "Update client last used timestamp",
            Self::GetAuthorizationCodeClientId => "Get client ID from authorization code",
            Self::DeleteRefreshToken => "Delete refresh token",
            Self::MarkAuthorizationCodeUsed => "Mark authorization code as used",
            Self::DeleteExpiredRefreshTokens => "Delete expired refresh tokens",
            Self::GetRoles => "Get user roles",
            Self::CheckRoleExists => "Check if role exists",
            Self::GetDefaultRoles => "Get default roles",
            Self::GetOAuthUser => "Get OAuth authenticated user details",
            Self::GetPlatformOverview => "Get platform overview",
            Self::GetTopUsers => "Get top users",
            Self::GetTopAgents => "Get top agents",
            Self::GetTopTools => "Get top tools",
            Self::GetActivityTrend => "Get activity trend",
            Self::GetUserMetrics => "Get user metrics",
            Self::GetCostBreakdown => "Get cost breakdown",
            Self::GetSystemHealth => "Get system health",
            Self::CreateAnalyticsSession => "Create analytics session",
            Self::GetAnalyticsSession => "Get analytics session",
            Self::SessionExists => "Check if session exists",
            Self::EndAnalyticsSession => "End analytics session",
            Self::MarkSessionAsScanner => "Mark session as scanner",
            Self::UpdateSessionQuality => "Update session quality",
            Self::UpdateSessionActivity => "Update session activity",
            Self::UpdateSessionEndpoints => "Update session endpoints",
            Self::RecordEndpointRequest => "Record endpoint request",
            Self::IncrementSessionAiUsage => "Increment session AI usage",
            Self::IncrementSessionTaskActivity => "Increment session task activity",
            Self::GetAnalyticsActiveSessions => "Get active analytics sessions",
            Self::CleanupInactiveAnalyticsSessions => "Cleanup inactive analytics sessions",
            Self::FindSessionByFingerprint => "Find session by fingerprint",
            Self::FindSessionByFingerprintAny => "Find any session by fingerprint (test helper)",
            Self::GetEndpointRequestsBySession => "Get endpoint requests for session",
            Self::CleanupExpiredAnonymousSessions => "Cleanup expired anonymous sessions",
            Self::MigrateSessionToUser => "Migrate session to registered user",
            Self::UpdateSessionContexts => "Update session contexts",
            Self::UpdateSessionUserAgentTasks => "Update user ID in agent tasks",
            Self::UpdateSessionUserTaskMessages => "Update user ID in task messages",
            Self::UpdateSessionUserAiRequests => "Update user ID in AI requests",
            Self::UpdateSessionUserLogs => "Update user ID in logs",
            Self::UpdateSessionUserToolExecutions => "Update user ID in tool executions",
            Self::DeleteTemporarySessionUser => "Delete temporary session user",
            Self::DeleteContextBySession => "Delete user context by session",
            Self::DeleteSessionById => "Delete session by ID",
            Self::RecordEvent => "Record analytics event",
            Self::GetEventsBySession => "Get events by session",
            Self::GetEventsSummary => "Get events summary",
            Self::GetDailyActivityTrendBase => "Get daily activity trend",
            Self::GetSystemHealthMetrics => "Get system health metrics",
            Self::GetAgentUsageAnalyticsBase => "Get agent usage analytics",
            Self::GetTopUsersSummary => "Get top users summary",
            Self::GetSessionQualityMetrics => "Get session quality metrics",
            Self::GetUserAnalyticsSummary => "Get user analytics summary",
            Self::CreateUser => "Create user",
            Self::GetUserById => "Get user by ID",
            Self::GetUserByName => "Get user by name",
            Self::GetUserByEmail => "Get user by email",
            Self::DeleteUser => "Delete user",
            Self::ListUsers => "List users",
            Self::SearchUsers => "Search users",
            Self::CountUsers => "Count users",
            Self::AssignRole => "Assign role to user",
            Self::RemoveRole => "Remove role from user",
            Self::GetUserRoles => "Get user roles",
            Self::FindByRole => "Find users by role",
            Self::FindFirstAdmin => "Find first admin user",
            Self::FindFirstUser => "Find first user",
            Self::CreateUserSession => "Create user session",
            Self::GetUserSession => "Get user session",
            Self::DeleteUserSession => "Delete user session",
            Self::ListUserSessions => "List user sessions",
            Self::CreateAnonymousUser => "Create anonymous user",
            Self::GetAnonymousUser => "Get anonymous user",
            Self::ConvertAnonymousUser => "Convert anonymous user",
            Self::DeleteAnonymousUser => "Delete anonymous user",
            Self::IsTemporaryAnonymous => "Check if user is temporary anonymous",
            Self::CleanupOldAnonymousUsers => "Cleanup old anonymous users",
            Self::UpdateUserEmail => "Update user email",
            Self::UpdateUserEmailFullName => "Update user email and full name",
            Self::UpdateUserEmailStatus => "Update user email and status",
            Self::UpdateUserFullName => "Update user full name",
            Self::UpdateUserFullNameStatus => "Update user full name and status",
            Self::UpdateUserStatus => "Update user status",
            Self::UpdateUserAllFields => "Update all user fields",
            Self::CreateContent => "Create content",
            Self::GetContentById => "Get content by ID",
            Self::GetContentByUrl => "Get content by URL",
            Self::GetContentBySourceAndSlug => "Get content by source and slug",
            Self::GetSocialContentByParent => "Get social content by parent blog post ID",
            Self::GetContentByVersionHash => "Get content by version hash",
            Self::ListContent => "List content",
            Self::ListAllContent => "List all content across all sources",
            Self::ListContentBySource => "List content by source",
            Self::SearchByCategory => "Search content by category",
            Self::SearchByTags => "Search content by tags",
            Self::SearchContentByKeyword => "Search content by keyword",
            Self::UpdateContent => "Update content",
            Self::UpdateContentImage => "Update content image",
            Self::DeleteContent => "Delete content",
            Self::DeleteContentBySource => "Delete content by source",
            Self::AddLinksColumnToContent => "Add links column to markdown_content table",
            Self::CreateTag => "Create tag",
            Self::GetTagById => "Get tag by ID",
            Self::GetTagByName => "Get tag by name",
            Self::ListTags => "List tags",
            Self::DeleteTag => "Delete tag",
            Self::LinkTagToContent => "Link tag to content",
            Self::UnlinkTagFromContent => "Unlink tag from content",
            Self::UnlinkAllTagsFromContent => "Unlink all tags from content",
            Self::GetTagsByContent => "Get tags by content",
            Self::CreateCategory => "Create category",
            Self::GetCategoryById => "Get category by ID",
            Self::GetCategoryByName => "Get category by name",
            Self::ListCategories => "List categories",
            Self::DeleteCategory => "Delete category",
            Self::InsertToolExecution => "Insert tool execution",
            Self::GetToolExecution => "Get tool execution",
            Self::ListToolExecutionsBySession => "List tool executions by session",
            Self::ListToolExecutionsByUser => "List tool executions by user",
            Self::UpdateToolExecutionStatus => "Update tool execution status",
            Self::UpdateToolExecutionResult => "Update tool execution result",
            Self::RegisterTool => "Register tool",
            Self::GetToolMetadata => "Get tool metadata",
            Self::ListAvailableTools => "List available tools",
            Self::UpdateToolMetadata => "Update tool metadata",
            Self::GetToolUsageStats => "Get tool usage stats",
            Self::GetToolErrorRate => "Get tool error rate",
            Self::GetToolPerformanceMetrics => "Get tool performance metrics",
            Self::RegisterMcpServer => "Register MCP server",
            Self::ListMcpServers => "List MCP servers",
            Self::UpdateMcpServerStatus => "Update MCP server status",
            Self::RemoveMcpServer => "Remove MCP server",
            Self::LinkToolToServer => "Link tool to MCP server",
            Self::GetToolsByServer => "Get tools by MCP server",
            Self::UnlinkToolFromServer => "Unlink tool from MCP server",
            Self::GetMcpConfig => "Get MCP config",
            Self::UpdateMcpConfig => "Update MCP config",
            Self::InsertAiRequest => "Insert AI request",
            Self::InsertAiImageRequest => "Insert AI image request",
            Self::GetAiRequest => "Get AI request",
            Self::ListAiRequestsBySession => "List AI requests by session",
            Self::ListAiRequestsByUser => "List AI requests by user",
            Self::UpdateAiRequestStatus => "Update AI request status",
            Self::InsertRequestMessage => "Insert AI request message",
            Self::InsertResponseMessage => "Insert AI response message",
            Self::GetAiMessageMaxSequence => "Get AI message max sequence",
            Self::InsertGeneratedImage => "Insert generated image",
            Self::GetGeneratedImageByUuid => "Get generated image by UUID",
            Self::ListGeneratedImagesByUser => "List generated images by user",
            Self::DeleteGeneratedImage => "Delete generated image",
            Self::GetRequestMessages => "Get AI request messages",
            Self::InsertToolCall => "Insert tool call",
            Self::GetToolCalls => "Get tool calls",
            Self::RegisterProvider => "Register AI provider",
            Self::GetProvider => "Get AI provider",
            Self::ListProviders => "List AI providers",
            Self::UpdateProviderConfig => "Update AI provider config",
            Self::RegisterModel => "Register AI model",
            Self::GetModel => "Get AI model",
            Self::ListModelsByProvider => "List AI models by provider",
            Self::UpdateModelCapabilities => "Update AI model capabilities",
            Self::GetTokenUsageByModel => "Get token usage by model",
            Self::GetUserAiUsageAll => "Get all user AI usage",
            Self::GetUserAiUsageWithDateRange => "Get user AI usage with date range",
            Self::GetUserAiUsageSinceDate => "Get user AI usage since date",
            Self::GetUserAiUsageUntilDate => "Get user AI usage until date",
            Self::GetProviderUsageAll => "Get provider usage for all users",
            Self::GetProviderUsageByUser => "Get provider usage by user",
            Self::CreateLog => "Create log",
            Self::GetLog => "Get log",
            Self::ListLogs => "List logs",
            Self::ListLogsPaginated => "List logs paginated",
            Self::DeleteLog => "Delete log",
            Self::DeleteOldLogs => "Delete logs older than 7 days",
            Self::LogAnalyticsEvent => "Log analytics event",
            Self::GetLogsByLevel => "Get logs by level",
            Self::GetLogsByModule => "Get logs by module",
            Self::GetLogsByUser => "Get logs by user",
            Self::GetLogsBySession => "Get logs by session",
            Self::GetLogStats => "Get log stats",
            Self::GetErrorRate => "Get error rate",
            Self::VacuumLogs => "Vacuum logs",
            Self::OptimizeLogIndices => "Optimize log indices",
            Self::GetLogRetentionMetrics => "Get log retention metrics",
            Self::ArchiveOldLogs => "Archive old logs",
            Self::CheckConfigTableExists => "Check if config table exists",
            Self::InsertModule => "Insert module config",
            Self::GetAllModules => "Get all modules",
            Self::EnableModule => "Enable module",
            Self::DisableModule => "Disable module",
            Self::DeleteModule => "Delete module",
            Self::UpdateModule => "Update module",
            Self::CreateVariable => "Create config variable",
            Self::GetVariable => "Get config variable by name",
            Self::GetVariableById => "Get config variable by ID",
            Self::ListVariables => "List config variables",
            Self::ListVariablesByCategory => "List config variables by category",
            Self::DeleteVariable => "Delete config variable",
            Self::UpdateVariable => "Update config variable",
            Self::ListActiveUserSessions => "List active user sessions",
            Self::ListRecentUserSessions => "List recent user sessions",
            Self::GetUserActivity => "Get user activity summary",
            Self::UpdateUserRoles => "Update user roles",
            Self::GetAgentConversationStats => "Get agent conversation statistics",
            Self::GetTopAgentsByConversations => "Get top agents by conversation count",
            Self::GetTrafficSummary => "Get traffic summary metrics",
            Self::GetDeviceBreakdown => "Get traffic breakdown by device",
            Self::GetGeoBreakdown => "Get traffic breakdown by geolocation",
            Self::GetClientBreakdown => "Get traffic breakdown by client",
            Self::GetVisitorJourney => {
                "Get visitor journey metrics (new vs returning, bounce rate)"
            },
            Self::GetTrafficSources => "Get traffic sources breakdown with engagement metrics",
            Self::GetUtmCampaigns => "Get UTM campaign performance metrics",
            Self::GetLandingPages => "Get landing page analysis with bounce rates",
            Self::GetBotScannerSummary => "Get bot and scanner traffic statistics",
            Self::GetScannerPaths => "Get paths accessed by scanners",
            Self::GetTrafficTrendHourly => "Get traffic trends by hour",
            Self::GetTrafficTrendDaily => "Get traffic trends by day",
            Self::GetConversationSummary => "Get conversation summary statistics",
            Self::GetConversationsByAgent => "Get conversations grouped by agent",
            Self::GetConversationsByStatus => "Get conversations grouped by status",
            Self::GetRecentConversations => "Get recent conversations with details",
            Self::GetRecentConversationsPaginated => {
                "Get paginated recent conversations with evaluation data"
            },
            Self::GetConversationTrends => "Get daily conversation trends with metrics",
            Self::GetConversationMetricsMultiPeriod => {
                "Get conversation counts across multiple time periods"
            },
            Self::GetTopSubjects => "Get top conversation subjects/topics",
            Self::GetSubjectTrends => "Get conversation subject trends over time",
            Self::AnalyzeConversation => "Analyze and store conversation subject",
            Self::GetTopContent => "Get top content by views or engagement",
            Self::GetCategoryPerformance => "Get content category performance metrics",
            Self::GetContentTrends => "Get content trends (new, trending, evergreen, declining)",
            Self::GetDailyViewsPerContent => "Get daily views per content for time series charting",
            Self::GetTopReferrers => "Get top referrer URLs and landing pages",
            Self::GetDeviceLocation => "Get device type and location distribution",
            Self::GetContentClickMetrics => "Get content click metrics and engagement data",
            Self::GetSessionClickEngagement => "Get session click engagement metrics",
            Self::UpsertScheduledJob => "Upsert scheduled job",
            Self::GetScheduledJob => "Get scheduled job by name",
            Self::ListEnabledJobs => "List all enabled scheduled jobs",
            Self::UpdateJobExecution => "Update job execution status",
            Self::IncrementJobRunCount => "Increment job run count",
            Self::CreateEvaluation => "Create conversation evaluation",
            Self::GetEvaluationByContext => "Get conversation evaluation by context ID",
            Self::GetEvaluationMetrics => "Get evaluation metrics for date range",
            Self::GetLowScoringConversations => "Get low-scoring conversations for review",
            Self::GetEvaluationQualityDistribution => "Get quality score distribution by buckets",
            Self::GetRecentEvaluations => "Get recent evaluations with details",
            Self::GetConversationsByLocation => {
                "Get conversation breakdown by location via sessions join"
            },
            Self::GetUnevaluatedConversations => "Get conversations without evaluations",
            Self::GetTopIssuesEncountered => "Get most frequently encountered issues",
            Self::GetGoalAchievementStats => "Get goal achievement statistics",
            Self::GetDetailedEvaluations => "Get detailed evaluations for table display",
            Self::FetchTraceEvents => "Fetch trace events by trace ID",
            Self::CliListTables => "List all database tables",
            Self::CliDescribeTable => "Describe table schema",
            Self::CliGetTableCount => "Get table row count",
            Self::CliGetDbVersion => "Get database version",
            Self::DeleteOrphanedLogs => "Delete orphaned log records",
            Self::DeleteOrphanedAnalyticsEvents => "Delete orphaned analytics events",
            Self::DeleteOrphanedMcpExecutions => "Delete orphaned MCP tool executions",
            Self::DeleteExpiredOAuthCodes => "Delete expired OAuth authorization codes",
            Self::DeleteExpiredOAuthTokens => "Delete expired OAuth refresh tokens",
            Self::CreateLink => "Create a new link for tracking",
            Self::GetLinkById => "Get link by ID",
            Self::GetLinkByShortCode => "Get link by short code",
            Self::ListLinksByCampaign => "List links by campaign",
            Self::ListLinksBySourceContent => "List links by source content",
            Self::IncrementLinkClicks => "Increment link click count",
            Self::RecordClick => "Record link click event",
            Self::GetClicksByLink => "Get clicks for a link",
            Self::CheckSessionClickedLink => "Check if session clicked a link",
            Self::GetLinkPerformance => "Get link performance metrics",
            Self::GetAggregatedLinkPerformance => "Get aggregated link performance by campaign",
            Self::GetCampaignPerformance => "Get campaign performance metrics",
            Self::GetContentJourneyMap => "Get content journey map",
        }
    }
}
