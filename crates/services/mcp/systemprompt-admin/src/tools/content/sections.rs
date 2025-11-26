use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use systemprompt_models::artifacts::{
    ChartDataset, ChartSectionData, Column, ColumnType, DashboardSection, LayoutWidth,
    SectionLayout, SectionType, TableArtifact, TableHints,
};

use super::models::{ContentPerformance, DailyViewData, Referrer};

pub fn create_top_content_section(content: &[ContentPerformance]) -> DashboardSection {
    let table = TableArtifact::new(vec![
        Column::new("title", ColumnType::String).with_header("TITLE"),
        Column::new("link", ColumnType::Link).with_header("URL"),
        Column::new("views", ColumnType::Integer).with_header("VIEWS"),
        Column::new("visitors", ColumnType::Integer).with_header("VISITORS"),
        Column::new("age_days", ColumnType::Integer).with_header("AGE (DAYS)"),
    ])
    .with_rows(
        content
            .iter()
            .map(|item| {
                json!({
                    "title": item.title.clone(),
                    "link": item.trackable_url.clone(),
                    "views": item.total_views,
                    "visitors": item.unique_visitors,
                    "age_days": item.days_old,
                })
            })
            .collect(),
    )
    .with_hints(
        TableHints::new()
            .with_sortable(vec![
                "views".to_string(),
                "visitors".to_string(),
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
            .push((view.view_date.clone(), view.daily_views as f64));
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
        totals.into_iter().take(5).collect()
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
