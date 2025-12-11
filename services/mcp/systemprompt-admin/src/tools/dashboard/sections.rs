use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use systemprompt_core_system::models::analytics::PlatformOverview;
use systemprompt_models::artifacts::{
    Column, ColumnType, DashboardSection, LayoutWidth, SectionLayout, SectionType, TableArtifact,
    TableHints,
};

use super::models::{
    AgentUsageRow, ConversationMetrics, DailyTrend, RecentConversation, ToolUsageRow,
    TrafficSummary,
};

pub fn create_realtime_activity_section(overview: &PlatformOverview) -> DashboardSection {
    let cards = vec![
        json!({
            "title": "Active Users (24h)",
            "value": overview.active_users_24h.to_string(),
            "subtitle": "last 24 hours",
            "icon": "users",
            "status": "success"
        }),
        json!({
            "title": "Active Sessions",
            "value": overview.active_sessions.to_string(),
            "subtitle": "current active",
            "icon": "activity"
        }),
        json!({
            "title": "Total Registered Users",
            "value": overview.total_users.to_string(),
            "subtitle": "all time",
            "icon": "user-check"
        }),
    ];

    DashboardSection::new(
        "realtime_activity",
        "Real-time Activity",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 1,
    })
}

pub fn create_conversations_overview_section(metrics: &ConversationMetrics) -> DashboardSection {
    let cards = vec![
        json!({
            "title": "Conversations (24h)",
            "value": metrics.conversations_24h.to_string(),
            "subtitle": calculate_trend(metrics.conversations_24h, metrics.conversations_prev_24h),
            "icon": "message-circle",
            "status": "info"
        }),
        json!({
            "title": "Conversations (7d)",
            "value": metrics.conversations_7d.to_string(),
            "subtitle": calculate_trend(metrics.conversations_7d, metrics.conversations_prev_7d),
            "icon": "message-square",
            "status": "info"
        }),
        json!({
            "title": "Conversations (30d)",
            "value": metrics.conversations_30d.to_string(),
            "subtitle": calculate_trend(metrics.conversations_30d, metrics.conversations_prev_30d),
            "icon": "messages",
            "status": "info"
        }),
    ];

    DashboardSection::new(
        "conversations_overview",
        "Conversations Overview",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 2,
    })
}

pub fn create_recent_conversations_section(
    conversations: &[RecentConversation],
) -> DashboardSection {
    let table = TableArtifact::new(vec![
        Column::new("context_id", ColumnType::String).with_header("Context ID"),
        Column::new("agent_name", ColumnType::String).with_header("Agent"),
        Column::new("started_at", ColumnType::String).with_header("Started"),
        Column::new("duration", ColumnType::String).with_header("Duration"),
        Column::new("status", ColumnType::String).with_header("Status"),
        Column::new("messages", ColumnType::Number).with_header("Messages"),
    ])
    .with_rows(
        conversations
            .iter()
            .map(|c| {
                let duration = c.task_completed_at.signed_duration_since(c.task_started_at);
                let duration_secs = duration.num_seconds() as f64;
                let duration_mins = duration_secs / 60.0;
                json!({
                    "context_id": &c.context_id[..8.min(c.context_id.len())],
                    "agent_name": c.agent_name,
                    "started_at": format_timestamp(&c.started_at),
                    "duration": format!("{:.1}m", duration_mins),
                    "status": c.status,
                    "messages": c.message_count
                })
            })
            .collect(),
    )
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "agent_name".to_string(),
                "started_at".to_string(),
                "duration".to_string(),
                "messages".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new(
        "recent_conversations",
        "Recent Conversations (newest first)",
        SectionType::Table,
    )
    .with_data(table.to_response())
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 3,
    })
}

pub fn create_traffic_summary_section(summary: &TrafficSummary) -> DashboardSection {
    let cards = vec![
        json!({
            "title": "Total Sessions",
            "value": summary.total_sessions.to_string(),
            "subtitle": "user sessions",
            "icon": "globe",
            "status": "info"
        }),
        json!({
            "title": "Total Requests",
            "value": summary.total_requests.to_string(),
            "subtitle": "API requests",
            "icon": "activity",
            "status": "info"
        }),
        json!({
            "title": "Unique Visitors",
            "value": summary.unique_users.to_string(),
            "subtitle": "distinct users",
            "icon": "users",
            "status": "info"
        }),
    ];

    DashboardSection::new(
        "traffic_overview",
        "Traffic Overview",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 4,
    })
}

pub fn create_daily_trends_section(trends: &[DailyTrend]) -> DashboardSection {
    let labels: Vec<String> = trends.iter().map(|t| t.date.clone()).collect();
    let conversations_data: Vec<i64> = trends.iter().map(|t| t.conversations).collect();
    let tool_executions_data: Vec<i64> = trends.iter().map(|t| t.tool_executions).collect();
    let active_users_data: Vec<i64> = trends.iter().map(|t| t.active_users).collect();

    DashboardSection::new("daily_trends", "Daily Trends", SectionType::Chart)
        .with_data(json!({
            "chart_type": "line",
            "labels": labels,
            "datasets": [
                {"label": "Conversations", "data": conversations_data},
                {"label": "Tool Executions", "data": tool_executions_data},
                {"label": "Active Users", "data": active_users_data}
            ]
        }))
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 5,
        })
}

pub fn create_agent_usage_section(agent_data: &[AgentUsageRow]) -> DashboardSection {
    let rows: Vec<serde_json::Value> = agent_data
        .iter()
        .map(|agent| {
            json!({
                "name": agent.agent_name,
                "h24": agent.h24,
                "d7": agent.d7,
                "d30": agent.d30
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("name", ColumnType::String).with_header("Agent"),
        Column::new("h24", ColumnType::Number).with_header("24h"),
        Column::new("d7", ColumnType::Number).with_header("7d"),
        Column::new("d30", ColumnType::Number).with_header("30d"),
    ])
    .with_rows(rows)
    .with_hints(
        TableHints::new()
            .with_sortable(vec!["h24".to_string(), "d7".to_string(), "d30".to_string()])
            .filterable(),
    );

    DashboardSection::new("agent_usage", "Agent Usage", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Half,
            order: 6,
        })
}

pub fn create_tool_usage_section(tool_data: &[ToolUsageRow]) -> DashboardSection {
    let rows: Vec<serde_json::Value> = tool_data
        .iter()
        .map(|tool| {
            json!({
                "name": tool.tool_name,
                "h24": tool.h24,
                "d7": tool.d7,
                "d30": tool.d30
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("name", ColumnType::String).with_header("Tool"),
        Column::new("h24", ColumnType::Number).with_header("24h"),
        Column::new("d7", ColumnType::Number).with_header("7d"),
        Column::new("d30", ColumnType::Number).with_header("30d"),
    ])
    .with_rows(rows)
    .with_hints(
        TableHints::new()
            .with_sortable(vec!["h24".to_string(), "d7".to_string(), "d30".to_string()])
            .filterable(),
    );

    DashboardSection::new("tool_usage", "Tool Usage", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Half,
            order: 7,
        })
}

fn calculate_trend(current: i64, previous: i64) -> String {
    if previous == 0 {
        if current > 0 {
            return "↑ new".to_string();
        }
        return "—".to_string();
    }
    let change = ((current - previous) as f64 / previous as f64) * 100.0;
    if change > 0.0 {
        format!("↑ {:.0}%", change)
    } else if change < 0.0 {
        format!("↓ {:.0}%", change.abs())
    } else {
        "—".to_string()
    }
}

fn format_timestamp(timestamp_str: &str) -> String {
    match timestamp_str.parse::<DateTime<Utc>>() {
        Ok(dt) => {
            let now = Utc::now();
            let diff = now.signed_duration_since(dt);

            if diff < Duration::zero() {
                dt.format("%b %d, %Y %H:%M UTC").to_string()
            } else if diff.num_seconds() < 60 {
                format!("{} seconds ago", diff.num_seconds())
            } else if diff.num_minutes() < 60 {
                let mins = diff.num_minutes();
                format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
            } else if diff.num_hours() < 24 {
                let hours = diff.num_hours();
                format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
            } else if diff.num_days() < 7 {
                let days = diff.num_days();
                format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
            } else {
                dt.format("%b %d, %Y %H:%M UTC").to_string()
            }
        }
        Err(_) => timestamp_str.to_string(),
    }
}
