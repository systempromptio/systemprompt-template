use serde_json::{json, Value as JsonValue};
use systemprompt_models::artifacts::{
    Column, ColumnType, DashboardSection, LayoutWidth, SectionLayout, SectionType, TableArtifact,
};

use super::models::{DeviceBreakdown, LandingPage, TrafficSource, TrafficSummary};

pub fn create_traffic_summary_section(summary: &TrafficSummary) -> DashboardSection {
    let cards = vec![
        json!({
            "title": "Total Sessions",
            "value": summary.total_sessions.to_string(),
            "icon": "sessions",
            "status": "success"
        }),
        json!({
            "title": "Total Requests",
            "value": summary.total_requests.to_string(),
            "icon": "activity",
            "status": "success"
        }),
        json!({
            "title": "Unique Users",
            "value": summary.unique_users.to_string(),
            "icon": "users",
            "status": "success"
        }),
        json!({
            "title": "Avg Session Duration",
            "value": format!("{:.1}s", summary.avg_session_duration_secs),
            "icon": "clock",
            "status": "info"
        }),
        json!({
            "title": "Avg Requests/Session",
            "value": format!("{:.1}", summary.avg_requests_per_session),
            "icon": "trending-up",
            "status": "info"
        }),
        json!({
            "title": "Total AI Cost",
            "value": format!("${:.4}", summary.total_cost_cents as f64 / 1_000_000.0),
            "icon": "dollar-sign",
            "status": "warning"
        }),
    ];

    DashboardSection::new(
        "traffic_summary",
        "Traffic Summary",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 1,
    })
}

pub fn create_traffic_table_section(
    sources: &[TrafficSource],
    pages: &[LandingPage],
    devices: &[DeviceBreakdown],
) -> DashboardSection {
    let rows: Vec<JsonValue> = sources
        .iter()
        .enumerate()
        .map(|(idx, source)| {
            let page = pages
                .get(idx)
                .map(|p| p.landing_page.clone())
                .unwrap_or_default();
            let device = devices
                .get(idx)
                .map(|d| d.device_type.clone())
                .unwrap_or_default();

            json!({
                "source": source.source_name,
                "landing_page": page,
                "traffic": source.session_count,
                "device": device,
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("source", ColumnType::String).with_header("Source"),
        Column::new("landing_page", ColumnType::String).with_header("Landing Page"),
        Column::new("traffic", ColumnType::Number).with_header("Traffic"),
        Column::new("device", ColumnType::String).with_header("Device"),
    ])
    .with_rows(rows);

    DashboardSection::new("traffic_table", "Traffic Overview", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 2,
        })
}
