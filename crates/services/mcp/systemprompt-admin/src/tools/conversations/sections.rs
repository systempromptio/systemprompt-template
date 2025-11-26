use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::JsonRow;
use systemprompt_models::artifacts::{
    Column, ColumnType, DashboardSection, LayoutWidth, SectionLayout, SectionType, TableArtifact,
    TableHints,
};

use super::models::{ConversationSummary, EvaluationStats, RecentConversation};

pub fn create_summary_cards_section(
    summary: &ConversationSummary,
    eval_stats: &EvaluationStats,
) -> DashboardSection {
    let cards = vec![
        json!({
            "title": "Total Conversations",
            "value": summary.total_conversations.to_string(),
            "icon": "message-square",
            "status": "success"
        }),
        json!({
            "title": "Total Messages",
            "value": summary.total_messages.to_string(),
            "icon": "chat",
            "status": "success"
        }),
        json!({
            "title": "Avg Messages/Conversation",
            "value": format!("{:.1}", summary.avg_messages_per_conversation),
            "icon": "trending-up",
            "status": "info"
        }),
        json!({
            "title": "Avg Execution Time",
            "value": format!("{:.0}ms", summary.avg_execution_time_ms),
            "icon": "clock",
            "status": "info"
        }),
        json!({
            "title": "Failed Conversations",
            "value": summary.failed_conversations.to_string(),
            "icon": "x-circle",
            "status": if summary.failed_conversations == 0 { "success" } else { "warning" }
        }),
        json!({
            "title": "Evaluated Conversations",
            "value": eval_stats.evaluated_conversations.to_string(),
            "icon": "check-circle",
            "status": "info"
        }),
        json!({
            "title": "Average Quality Score",
            "value": format!("{:.0}/100", eval_stats.avg_quality_score),
            "icon": "star",
            "status": get_quality_status(eval_stats.avg_quality_score)
        }),
        json!({
            "title": "Goal Achievement Rate",
            "value": format!("{:.1}%", eval_stats.goal_achievement_rate),
            "icon": "target",
            "status": get_goal_status(eval_stats.goal_achievement_rate)
        }),
        json!({
            "title": "Avg User Satisfaction",
            "value": format!("{:.0}/100", eval_stats.avg_user_satisfaction),
            "icon": "thumbs-up",
            "status": get_satisfaction_status(eval_stats.avg_user_satisfaction)
        }),
    ];

    DashboardSection::new(
        "conversation_summary",
        "Summary Metrics",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 2,
    })
}

pub fn create_conversations_table_section(
    conversations: &[RecentConversation],
    page: i32,
    per_page: i32,
) -> DashboardSection {
    let table = TableArtifact::new(vec![
        Column::new("user", ColumnType::String).with_header("User"),
        Column::new("agent_name", ColumnType::String).with_header("Agent"),
        Column::new("started_at", ColumnType::String).with_header("Started"),
        Column::new("messages", ColumnType::Number).with_header("Messages"),
        Column::new("summary", ColumnType::String).with_header("Summary"),
    ])
    .with_rows(
        conversations
            .iter()
            .map(|conv| {
                let summary_text = conv.evaluation_summary.as_deref().unwrap_or("-");

                json!({
                    "user": &conv.user_name,
                    "agent_name": &conv.agent_name,
                    "started_at": conv.started_at_formatted.as_deref().unwrap_or(&conv.started_at),
                    "messages": conv.message_count,
                    "summary": summary_text,
                })
            })
            .collect(),
    )
    .with_hints(TableHints::new().filterable());

    let title = format!(
        "Recent Conversations (Page {} - {} per page)",
        page, per_page
    );

    DashboardSection::new("recent_conversations", &title, SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 1,
        })
}

pub fn create_conversation_trends_section(trends: &[JsonRow]) -> DashboardSection {
    let row = trends.first();

    let cards: Vec<JsonValue> = if let Some(row) = row {
        vec![
            json!({
                "title": "Conversations (24h)",
                "value": row.get("conversations_24h")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    .to_string(),
                "subtitle": "Last 24 hours",
                "icon": "trending-up",
                "status": "info"
            }),
            json!({
                "title": "Conversations (7d)",
                "value": row.get("conversations_7d")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    .to_string(),
                "subtitle": "Last 7 days",
                "icon": "trending-up",
                "status": "info"
            }),
            json!({
                "title": "Conversations (30d)",
                "value": row.get("conversations_30d")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
                    .to_string(),
                "subtitle": "Last 30 days",
                "icon": "trending-up",
                "status": "info"
            }),
        ]
    } else {
        vec![]
    };

    DashboardSection::new(
        "conversation_tracking",
        "Conversation Tracking (Daily, Weekly, Monthly)",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 3,
    })
}

pub fn create_agent_breakdown_section(agent_data: &[JsonRow]) -> DashboardSection {
    let cards = agent_data
        .iter()
        .map(|row| {
            let agent_name = row
                .get("agent_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let count = row
                .get("conversation_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            let percentage = row
                .get("percentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            json!({
                "title": agent_name,
                "value": count.to_string(),
                "subtitle": format!("{:.1}% of all conversations", percentage),
                "icon": "cpu",
                "status": "info"
            })
        })
        .collect::<Vec<_>>();

    DashboardSection::new(
        "by_agent",
        "Conversations by Agent",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 4,
    })
}

fn get_quality_status(score: f64) -> &'static str {
    if score >= 80.0 {
        "success"
    } else if score >= 60.0 {
        "info"
    } else if score >= 40.0 {
        "warning"
    } else {
        "error"
    }
}

fn get_goal_status(rate: f64) -> &'static str {
    if rate >= 70.0 {
        "success"
    } else if rate >= 50.0 {
        "info"
    } else if rate >= 30.0 {
        "warning"
    } else {
        "error"
    }
}

fn get_satisfaction_status(score: f64) -> &'static str {
    if score >= 80.0 {
        "success"
    } else if score >= 60.0 {
        "info"
    } else if score >= 40.0 {
        "warning"
    } else {
        "error"
    }
}
