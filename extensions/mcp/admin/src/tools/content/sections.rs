use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use systemprompt_models::artifacts::{
    ChartDataset, ChartSectionData, Column, ColumnType, DashboardSection, LayoutWidth,
    SectionLayout, SectionType, TableArtifact, TableHints,
};

use super::models::{ContentPerformance, DailyViewData, Referrer, TrafficSummary};

pub fn create_traffic_summary_cards(summary: &TrafficSummary) -> DashboardSection {
    let format_trend = |diff: i32, percent: f64| -> String {
        let sign = if diff >= 0 { "+" } else { "" };
        format!("{sign}{diff} visitors ({percent:+.1}%)")
    };

    let cards = vec![
        json!({
            "title": "Visitors Today",
            "value": summary.traffic_1d.to_string(),
            "icon": "activity",
            "status": "success",
            "trend": format_trend(summary.diff_1d(), summary.percent_change_1d())
        }),
        json!({
            "title": "Visitors 7 Days",
            "value": summary.traffic_7d.to_string(),
            "icon": "trending-up",
            "status": "success",
            "trend": format_trend(summary.diff_7d(), summary.percent_change_7d())
        }),
        json!({
            "title": "Visitors 30 Days",
            "value": summary.traffic_30d.to_string(),
            "icon": "bar-chart",
            "status": "success",
            "trend": format_trend(summary.diff_30d(), summary.percent_change_30d())
        }),
    ];

    DashboardSection::new(
        "traffic_summary_cards",
        "Visitor Summary",
        SectionType::MetricsCards,
    )
    .with_data(json!({ "cards": cards }))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 0,
    })
}

pub fn create_top_content_section(content: &[ContentPerformance]) -> DashboardSection {
    let table = TableArtifact::new(vec![
        Column::new("title", ColumnType::String).with_header("TITLE"),
        Column::new("link", ColumnType::Link).with_header("URL"),
        Column::new("visitors_1d", ColumnType::Integer).with_header("1D"),
        Column::new("visitors_7d", ColumnType::Integer).with_header("7D"),
        Column::new("visitors_30d", ColumnType::Integer).with_header("30D"),
        Column::new("visitors_all_time", ColumnType::String).with_header("ALL TIME"),
        Column::new("age_days", ColumnType::Integer).with_header("AGE (DAYS)"),
    ])
    .with_rows(
        content
            .iter()
            .map(|item| {
                json!({
                    "title": item.title.clone(),
                    "link": item.trackable_url.clone(),
                    "visitors_1d": item.visitors_1d,
                    "visitors_7d": item.visitors_7d,
                    "visitors_30d": item.visitors_30d,
                    "visitors_all_time": format!("{} ({} views)", item.visitors_all_time, item.total_views),
                    "age_days": item.days_old,
                })
            })
            .collect(),
    )
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "visitors_1d".to_string(),
                "visitors_7d".to_string(),
                "visitors_30d".to_string(),
                "age_days".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new("top_content", "TOP PERFORMING CONTENT", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 1,
        })
}

pub fn create_daily_views_chart(daily_views: &[DailyViewData]) -> DashboardSection {
    let mut content_map: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    let mut all_dates: BTreeSet<String> = BTreeSet::new();

    for view in daily_views {
        all_dates.insert(view.view_date.clone());
        content_map
            .entry(view.title.clone())
            .or_default()
            .push((view.view_date.clone(), f64::from(view.daily_views)));
    }

    let top_content_with_totals: Vec<(String, i32)> = {
        let mut totals: Vec<(String, i32)> = content_map
            .iter()
            .map(|(title, views)| {
                let total: i32 = views.iter().map(|(_, v)| *v as i32).sum();
                (title.clone(), total)
            })
            .collect();
        totals.sort_by(|a, b| b.1.cmp(&a.1));
        totals.into_iter().take(10).collect()
    };

    let dates: Vec<String> = all_dates.into_iter().collect();

    let datasets: Vec<ChartDataset> = top_content_with_totals
        .iter()
        .filter_map(|(title, _total)| {
            let views = content_map.get(title)?;
            let views_map: HashMap<String, f64> = views.iter().cloned().collect();

            let data: Vec<f64> = dates
                .iter()
                .map(|date| *views_map.get(date).unwrap_or(&0.0))
                .collect();

            Some(ChartDataset::new(title.clone(), data))
        })
        .collect();

    let chart_data = ChartSectionData::new("line", dates, datasets);

    DashboardSection::new(
        "daily_views_chart",
        "DAILY VIEWS TIMELINE",
        SectionType::Chart,
    )
    .with_data(json!(chart_data))
    .with_layout(SectionLayout {
        width: LayoutWidth::Full,
        order: 0,
    })
}

pub fn create_top_referrers_section(referrers: &[Referrer]) -> DashboardSection {
    let table = TableArtifact::new(vec![
        Column::new("referrer_url", ColumnType::String).with_header("REFERRER URL"),
        Column::new("sessions", ColumnType::Integer).with_header("SESSIONS"),
        Column::new("unique_visitors", ColumnType::Integer).with_header("VISITORS"),
        Column::new("avg_pages_per_session", ColumnType::Number).with_header("AVG PAGES"),
    ])
    .with_rows(
        referrers
            .iter()
            .map(|item| {
                json!({
                    "referrer_url": item.referrer_url.clone(),
                    "sessions": item.sessions,
                    "unique_visitors": item.unique_visitors,
                    "avg_pages_per_session": item.avg_pages_per_session,
                })
            })
            .collect(),
    )
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "sessions".to_string(),
                "unique_visitors".to_string(),
                "avg_pages_per_session".to_string(),
            ])
            .filterable(),
    );

    DashboardSection::new("top_referrers", "TOP REFERRER URLS", SectionType::Table)
        .with_data(table.to_response())
        .with_layout(SectionLayout {
            width: LayoutWidth::Full,
            order: 2,
        })
}
