use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Core module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        GetPlatformOverview => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_platform_overview.sql"
        )),
        GetTopUsers => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_top_users.sql"
        )),
        GetTopAgents => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_top_agents.sql"
        )),
        GetTopTools => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_top_tools.sql"
        )),
        GetActivityTrend => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_activity_trend.sql"
        )),
        GetUserMetrics => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_user_metrics.sql"
        )),
        GetCostBreakdown => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_cost_breakdown.sql"
        )),
        GetSystemHealth => Some(include_str!(
            "../../../../core/src/queries/analytics/core_stats/postgres/get_system_health.sql"
        )),
        CreateAnalyticsSession => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/create_session.sql"
        )),
        GetAnalyticsSession => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/get_session.sql"
        )),
        SessionExists => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/session_exists.sql"
        )),
        EndAnalyticsSession => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/end_session.sql"
        )),
        MarkSessionAsScanner => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/mark_session_as_scanner.sql"
        )),
        UpdateSessionActivity => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_session_activity.sql"
        )),
        UpdateSessionEndpoints => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_endpoints.sql"
        )),
        RecordEndpointRequest => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/record_endpoint_request.sql"
        )),
        IncrementSessionAiUsage => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/increment_ai_usage.sql"
        )),
        IncrementSessionTaskActivity => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/increment_task_activity.sql"
        )),
        GetAnalyticsActiveSessions => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/get_active_sessions.sql"
        )),
        CleanupInactiveAnalyticsSessions => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/cleanup_inactive_sessions.sql"
        )),
        FindSessionByFingerprint => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/find_session_by_fingerprint.sql"
        )),
        FindSessionByFingerprintAny => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/find_session_by_fingerprint_any.sql"
        )),
        GetEndpointRequestsBySession => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/get_endpoint_requests_by_session.sql"
        )),
        CleanupExpiredAnonymousSessions => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/cleanup_expired_anonymous_select.sql"
        )),
        MigrateSessionToUser => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/migrate_session.sql"
        )),
        UpdateSessionContexts => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_contexts.sql"
        )),
        UpdateSessionUserAgentTasks => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_user_agent_tasks.sql"
        )),
        UpdateSessionUserTaskMessages => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_user_task_messages.sql"
        )),
        UpdateSessionUserAiRequests => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_user_ai_requests.sql"
        )),
        UpdateSessionUserLogs => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_user_logs.sql"
        )),
        UpdateSessionUserToolExecutions => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/update_user_tool_executions.sql"
        )),
        DeleteTemporarySessionUser => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/delete_temporary_user.sql"
        )),
        DeleteContextBySession => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/delete_context_by_session.sql"
        )),
        DeleteSessionById => Some(include_str!(
            "../../../../core/src/queries/analytics/session/postgres/delete_session_by_id.sql"
        )),
        RecordEvent => Some(include_str!(
            "../../../../core/src/queries/analytics/events/postgres/log_event.sql"
        )),
        GetEventsBySession => Some(include_str!(
            "../../../../core/src/queries/analytics/events/postgres/get_events_base.sql"
        )),
        GetEventsSummary => Some(include_str!(
            "../../../../core/src/queries/analytics/events/postgres/get_error_summary.sql"
        )),
        GetDailyActivityTrendBase => Some(include_str!(
            "../../../../core/src/queries/analytics/queries/postgres/get_daily_activity_trend_base.sql"
        )),
        GetSystemHealthMetrics => Some(include_str!(
            "../../../../core/src/queries/analytics/queries/postgres/get_system_health_metrics.sql"
        )),
        GetAgentUsageAnalyticsBase => Some(include_str!(
            "../../../../core/src/queries/analytics/queries/postgres/get_agent_usage_analytics_base.sql"
        )),
        GetTopUsersSummary => Some(include_str!(
            "../../../../core/src/queries/analytics/queries/postgres/get_top_users_summary.sql"
        )),
        GetSessionQualityMetrics => Some(include_str!(
            "../../../../core/src/queries/analytics/queries/postgres/get_session_quality_metrics.sql"
        )),
        GetUserAnalyticsSummary => Some(include_str!(
            "../../../../core/src/queries/analytics/queries/postgres/get_user_analytics_summary.sql"
        )),
        GetTrafficSummary => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_traffic_summary.sql"
        )),
        GetDeviceBreakdown => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_device_breakdown.sql"
        )),
        GetGeoBreakdown => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_geo_breakdown.sql"
        )),
        GetClientBreakdown => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_client_breakdown.sql"
        )),
        GetVisitorJourney => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_visitor_journey.sql"
        )),
        GetTrafficSources => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_traffic_sources.sql"
        )),
        GetUtmCampaigns => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_utm_campaigns.sql"
        )),
        GetLandingPages => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_landing_pages.sql"
        )),
        GetBotScannerSummary => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_bot_scanner_summary.sql"
        )),
        GetScannerPaths => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_scanner_paths.sql"
        )),
        GetTrafficTrendHourly => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_traffic_trend_hourly.sql"
        )),
        GetTrafficTrendDaily => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_traffic_trend_daily.sql"
        )),
        GetConversationSummary => Some(include_str!(
            "../../../../core/src/queries/analytics/conversations/postgres/get_conversation_summary.sql"
        )),
        GetConversationsByAgent => Some(include_str!(
            "../../../../core/src/queries/analytics/conversations/postgres/get_conversations_by_agent.sql"
        )),
        GetConversationsByStatus => Some(include_str!(
            "../../../../core/src/queries/analytics/conversations/postgres/get_conversations_by_status.sql"
        )),
        GetRecentConversations => Some(include_str!(
            "../../../../core/src/queries/analytics/conversations/postgres/get_recent_conversations.sql"
        )),
        GetRecentConversationsPaginated => Some(include_str!(
            "../../../../core/src/queries/analytics/conversations/postgres/get_recent_conversations_paginated.sql"
        )),
        GetConversationTrends => Some(include_str!(
            "../../../../core/src/queries/analytics/conversations/postgres/get_conversation_trends.sql"
        )),
        GetConversationMetricsMultiPeriod => Some(include_str!(
            "../../../../core/src/queries/analytics/conversations/postgres/get_conversation_metrics_multi_period.sql"
        )),
        GetTopSubjects => Some(include_str!(
            "../../../../core/src/queries/analytics/subjects/postgres/get_top_subjects.sql"
        )),
        GetSubjectTrends => Some(include_str!(
            "../../../../core/src/queries/analytics/subjects/postgres/get_subject_trends.sql"
        )),
        AnalyzeConversation => Some(include_str!(
            "../../../../core/src/queries/analytics/subjects/postgres/analyze_conversation.sql"
        )),
        FetchTraceEvents => Some(include_str!(
            "../../../../core/src/queries/cli/postgres/fetch_trace_events.sql"
        )),
        CliListTables => Some(include_str!(
            "../../../../core/src/queries/cli/postgres/list_tables.sql"
        )),
        CliDescribeTable => Some(include_str!(
            "../../../../core/src/queries/cli/postgres/describe_table.sql"
        )),
        CliGetTableCount => Some(include_str!(
            "../../../../core/src/queries/cli/postgres/get_table_count.sql"
        )),
        CliGetDbVersion => Some(include_str!(
            "../../../../core/src/queries/cli/postgres/get_db_version.sql"
        )),
        DeleteOrphanedLogs => Some(include_str!(
            "../../../../core/src/queries/cleanup/postgres/delete_orphaned_logs.sql"
        )),
        DeleteOrphanedAnalyticsEvents => Some(include_str!(
            "../../../../core/src/queries/cleanup/postgres/delete_orphaned_analytics_events.sql"
        )),
        DeleteOrphanedMcpExecutions => Some(include_str!(
            "../../../../core/src/queries/cleanup/postgres/delete_orphaned_mcp_executions.sql"
        )),
        DeleteOldLogs => Some(include_str!(
            "../../../../core/src/queries/cleanup/postgres/delete_old_logs.sql"
        )),
        DeleteExpiredOAuthCodes => Some(include_str!(
            "../../../../core/src/queries/cleanup/postgres/delete_expired_oauth_codes.sql"
        )),
        DeleteExpiredOAuthTokens => Some(include_str!(
            "../../../../core/src/queries/cleanup/postgres/delete_expired_oauth_tokens.sql"
        )),
        GetSessionClickEngagement => Some(include_str!(
            "../../../../core/src/queries/analytics/traffic/postgres/get_session_click_engagement.sql"
        )),

        _ => None,
    }
}
