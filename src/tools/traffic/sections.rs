use serde_json::{json, Value as JsonValue};
use systemprompt_models::artifacts::{
    Column, ColumnType, DashboardSection, LayoutWidth, SectionLayout, SectionType, TableArtifact,
    TableHints,
};

use super::models::{
    BrowserBreakdown, DeviceBreakdownWithTrends, GeographicBreakdown, OsBreakdown, Referrer,
    TrafficSummary,
};

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
            "value": format!("${:.4}", f64::from(summary.total_cost_cents) / 1_000_000.0),
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

pub fn create_device_breakdown_section(devices: &[DeviceBreakdownWithTrends]) -> DashboardSection {
    let rows: Vec<JsonValue> = devices
        .iter()
        .map(|device| {
            json!({
                "device": device.device_type,
                "sessions": device.sessions,
                "percentage": format!("{:.1}%", device.percentage),
                "1d": device.traffic_1d,
                "7d": device.traffic_7d,
                "30d": device.traffic_30d,
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("device", ColumnType::String).with_header("Device"),
        Column::new("sessions", ColumnType::Integer).with_header("Sessions"),
        Column::new("percentage", ColumnType::String).with_header("%"),
        Column::new("1d", ColumnType::Integer).with_header("1d"),
        Column::new("7d", ColumnType::Integer).with_header("7d"),
        Column::new("30d", ColumnType::Integer).with_header("30d"),
    ])
    .with_rows(rows)
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "sessions".to_string(),
                "1d".to_string(),
                "7d".to_string(),
                "30d".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new("device_breakdown", "Device Breakdown", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 2,
        })
}

pub fn create_geographic_breakdown_section(countries: &[GeographicBreakdown]) -> DashboardSection {
    let rows: Vec<JsonValue> = countries
        .iter()
        .map(|country| {
            json!({
                "country": country.country,
                "sessions": country.sessions,
                "percentage": format!("{:.1}%", country.percentage),
                "1d": country.traffic_1d,
                "7d": country.traffic_7d,
                "30d": country.traffic_30d,
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("country", ColumnType::String).with_header("Country"),
        Column::new("sessions", ColumnType::Integer).with_header("Sessions"),
        Column::new("percentage", ColumnType::String).with_header("%"),
        Column::new("1d", ColumnType::Integer).with_header("1d"),
        Column::new("7d", ColumnType::Integer).with_header("7d"),
        Column::new("30d", ColumnType::Integer).with_header("30d"),
    ])
    .with_rows(rows)
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "sessions".to_string(),
                "1d".to_string(),
                "7d".to_string(),
                "30d".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new(
        "geographic_breakdown",
        "Geographic Breakdown",
        SectionType::Table,
    )
    .with_data(table.to_response())
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 3,
    })
}

pub fn create_browser_breakdown_section(browsers: &[BrowserBreakdown]) -> DashboardSection {
    let rows: Vec<JsonValue> = browsers
        .iter()
        .map(|browser| {
            json!({
                "browser": browser.browser,
                "sessions": browser.sessions,
                "percentage": format!("{:.1}%", browser.percentage),
                "1d": browser.traffic_1d,
                "7d": browser.traffic_7d,
                "30d": browser.traffic_30d,
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("browser", ColumnType::String).with_header("Browser"),
        Column::new("sessions", ColumnType::Integer).with_header("Sessions"),
        Column::new("percentage", ColumnType::String).with_header("%"),
        Column::new("1d", ColumnType::Integer).with_header("1d"),
        Column::new("7d", ColumnType::Integer).with_header("7d"),
        Column::new("30d", ColumnType::Integer).with_header("30d"),
    ])
    .with_rows(rows)
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "sessions".to_string(),
                "1d".to_string(),
                "7d".to_string(),
                "30d".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new("browser_breakdown", "Browser Breakdown", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 4,
        })
}

pub fn create_os_breakdown_section(os_list: &[OsBreakdown]) -> DashboardSection {
    let rows: Vec<JsonValue> = os_list
        .iter()
        .map(|os| {
            json!({
                "os": os.os,
                "sessions": os.sessions,
                "percentage": format!("{:.1}%", os.percentage),
                "1d": os.traffic_1d,
                "7d": os.traffic_7d,
                "30d": os.traffic_30d,
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("os", ColumnType::String).with_header("OS"),
        Column::new("sessions", ColumnType::Integer).with_header("Sessions"),
        Column::new("percentage", ColumnType::String).with_header("%"),
        Column::new("1d", ColumnType::Integer).with_header("1d"),
        Column::new("7d", ColumnType::Integer).with_header("7d"),
        Column::new("30d", ColumnType::Integer).with_header("30d"),
    ])
    .with_rows(rows)
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "sessions".to_string(),
                "1d".to_string(),
                "7d".to_string(),
                "30d".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new(
        "os_breakdown",
        "Operating System Breakdown",
        SectionType::Table,
    )
    .with_data(table.to_response())
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 5,
    })
}

pub fn create_top_referrers_section(referrers: &[Referrer]) -> DashboardSection {
    let rows: Vec<JsonValue> = referrers
        .iter()
        .map(|item| {
            json!({
                "referrer_url": item.referrer_url.clone(),
                "sessions": item.sessions,
                "unique_visitors": item.unique_visitors,
                "avg_pages": format!("{:.1}", item.avg_pages_per_session),
                "avg_duration": format!("{:.0}s", item.avg_duration_sec),
            })
        })
        .collect();

    let table = TableArtifact::new(vec![
        Column::new("referrer_url", ColumnType::String).with_header("REFERRER URL"),
        Column::new("sessions", ColumnType::Integer).with_header("SESSIONS"),
        Column::new("unique_visitors", ColumnType::Integer).with_header("VISITORS"),
        Column::new("avg_pages", ColumnType::String).with_header("AVG PAGES"),
        Column::new("avg_duration", ColumnType::String).with_header("AVG DURATION"),
    ])
    .with_rows(rows)
    .with_hints(
        TableHints::new()
            .with_sortable(vec!["sessions".to_string(), "unique_visitors".to_string()])
            .filterable(),
    );

    DashboardSection::new("top_referrers", "TOP REFERRER URLS", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 6,
        })
}
