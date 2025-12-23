use serde_json::json;
use systemprompt_models::artifacts::{
    Column, ColumnType, DashboardSection, LayoutWidth, SectionLayout, SectionType, TableArtifact,
    TableHints,
};

use super::models::{LogEntry, LogStats};

pub fn create_stats_section(stats: &LogStats) -> DashboardSection {
    let cards = vec![
        json!({
            "title": "Total Logs",
            "value": stats.total_logs.to_string(),
            "subtitle": "all logs in system",
            "icon": "logs",
            "status": "info"
        }),
        json!({
            "title": "Errors",
            "value": stats.error_count.to_string(),
            "subtitle": "error level logs",
            "icon": "alert-circle",
            "status": if stats.error_count > 0 { "error" } else { "success" }
        }),
        json!({
            "title": "Warnings",
            "value": stats.warn_count.to_string(),
            "subtitle": "warning level logs",
            "icon": "alert-triangle",
            "status": if stats.warn_count > 0 { "warning" } else { "success" }
        }),
        json!({
            "title": "Unique Modules",
            "value": stats.unique_modules.to_string(),
            "subtitle": "modules generating logs",
            "icon": "package",
            "status": "info"
        }),
    ];

    DashboardSection::new("log_stats", "Log Statistics", SectionType::MetricsCards)
        .with_data(json!({ "cards": cards }))
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 1,
        })
}

pub fn create_logs_table_section(logs: &[LogEntry], page: i32) -> DashboardSection {
    let table = TableArtifact::new(vec![
        Column::new("timestamp", ColumnType::String).with_header("Time"),
        Column::new("level", ColumnType::String).with_header("Level"),
        Column::new("module", ColumnType::String).with_header("Module"),
        Column::new("message", ColumnType::String).with_header("Message"),
        Column::new("user_id", ColumnType::String).with_header("User"),
        Column::new("session_id", ColumnType::String).with_header("Session"),
    ])
    .with_rows(
        logs.iter()
            .map(|log| {
                json!({
                    "timestamp": format_timestamp_as_readable(&log.timestamp),
                    "level": log.level,
                    "module": log.module,
                    "message": log.message,
                    "user_id": log.user_id.as_deref().unwrap_or("N/A"),
                    "session_id": log.session_id.as_deref().unwrap_or("N/A"),
                })
            })
            .collect(),
    )
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "timestamp".to_string(),
                "level".to_string(),
                "module".to_string(),
                "message".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new(
        "recent_logs",
        format!("Recent Logs (Page {})", page + 1),
        SectionType::Table,
    )
    .with_data(table.to_response())
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 2,
    })
}

fn format_timestamp_as_readable(timestamp: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(timestamp).map_or_else(|_| timestamp.to_string(), |parsed| parsed.format("%b %d, %Y %H:%M:%S").to_string())
}
